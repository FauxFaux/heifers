use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::collections::HashSet;
use std::io;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;

use bitreader::BitReader;
use cast::u32;
use cast::u64;
use cast::usize;
use failure::Error;

use hevc;
use hevc::pps;
use hevc::sps;
use mpeg;
use mpeg::iprp::Property;
use mpeg::Extent;
use mpeg::FourCc;
use mpeg::ItemInfo;
use mpeg::ItemLoc;

// TODO: not Debug
#[derive(Debug)]
pub struct Heif {
    handler: FourCc,
    primary_item: u32,
    items: HashMap<u32, Item>,
    props: Vec<(HashSet<u32>, Property)>,
}

#[derive(Clone, Debug)]
struct Item {
    info: ItemInfo,
    location: ItemLoc,
}

impl Heif {
    pub fn new<R: Read>(from: R) -> Result<Heif, Error> {
        let raw = mpeg::load_meta(from)?;

        let handler = *get_only_element(&raw.handler)?;
        let primary_item = u32(*get_only_element(&raw.primary_item)?);

        let mut locators = HashMap::with_capacity(4 * raw.item_locators.len());
        for item_locators in raw.item_locators {
            for locator in item_locators {
                match locators.entry(locator.id) {
                    Entry::Occupied(_) => bail!("duplicate locator for {}", locator.id),
                    Entry::Vacant(vacant) => vacant.insert(locator),
                };
            }
        }

        let mut props = Vec::with_capacity(8 * locators.len());
        for prop_containers in raw.item_props {
            let container = get_only_element(prop_containers.containers)?;
            let assocs: Vec<mpeg::iprp::ItemPropertyAssociation> =
                get_only_element(prop_containers.associations)?;

            for (index, prop) in container.into_iter().enumerate() {
                let mut associated_items = HashSet::with_capacity(assocs.len());
                for item_assoc in &assocs {
                    if item_assoc
                        .associations
                        .iter()
                        .find(|x| usize(x.property_index) == index + 1)
                        .is_some()
                    {
                        associated_items.insert(item_assoc.item_id);
                    }
                }

                // TODO: we're losing the essential status here
                props.push((associated_items, prop));
            }
        }

        let mut items = HashMap::new();
        for item_infos in raw.item_infos {
            for info in item_infos {
                let id = u32(info.id);
                match items.entry(id) {
                    Entry::Occupied(_) => bail!("duplicate item id: {}", id),
                    Entry::Vacant(vacancy) => vacancy.insert(Item {
                        info,
                        location: locators
                            .remove(&id)
                            .ok_or_else(|| format_err!("no locator for item {}", id))?,
                    }),
                };
            }
        }

        ensure!(
            items.contains_key(&primary_item),
            "primary item has no data"
        );

        Ok(Heif {
            handler,
            primary_item,
            items,
            props,
        })
    }

    pub fn primary_item_id(&self) -> u32 {
        self.primary_item
    }

    pub fn open_item_data<R: Read + Seek>(
        &self,
        mut from: R,
        item: u32,
    ) -> Result<Extents<R>, Error> {
        let item = self
            .items
            .get(&item)
            .ok_or_else(|| format_err!("invalid item id"))?;

        let first_extent = &item
            .location
            .extents
            .get(0)
            .ok_or_else(|| format_err!("empty extents"))?;

        ensure!(0 != first_extent.length, "empty first extent");
        ensure!(0 == first_extent.index, "empty first extent");

        from.seek(SeekFrom::Start(
            item.location.base_offset + first_extent.offset,
        ))?;

        Ok(Extents {
            inner: from,
            base: item.location.base_offset,
            extents: &item.location.extents,
            current_extent: 0,
            current_pos: 0,
        })
    }

    pub fn find_pps(&self, item: u32) -> Result<pps::PicParamSet, Error> {
        for (ids, prop) in &self.props {
            if !ids.contains(&item) {
                continue;
            }

            if let Property::HvcCodecSettings(hvcc) = prop {
                for nal in &hvcc.nals {
                    if hevc::NAL_PPS_NUT == nal.completeness_and_nal_unit_type {
                        ensure!(1 == nal.units.len(), "expecting only one unit");
                        let bytes = &nal.units[0];
                        // TODO: validate NAL unit header, 2..
                        return Ok(pps::picture_parameter_set(&mut BitReader::new(
                            &bytes[2..],
                        ))?);
                    }
                }
            }
        }

        bail!("not found");
    }

    // TODO: generic?
    pub fn find_sps(&self, item: u32) -> Result<sps::SeqParamSet, Error> {
        for (ids, prop) in &self.props {
            if !ids.contains(&item) {
                continue;
            }

            if let Property::HvcCodecSettings(hvcc) = prop {
                for nal in &hvcc.nals {
                    if hevc::NAL_SPS_NUT == nal.completeness_and_nal_unit_type {
                        ensure!(1 == nal.units.len(), "expecting only one unit");
                        let bytes = &nal.units[0];
                        // TODO: validate NAL unit header, 2..
                        return Ok(sps::seq_parameter_set(&mut BitReader::new(&bytes[2..]))?);
                    }
                }
            }
        }

        bail!("not found");
    }
}

pub struct Extents<'h, R> {
    inner: R,
    base: u64,
    extents: &'h [Extent],
    current_extent: usize,
    current_pos: u64,
}

impl<'h, R: Read + Seek> Read for Extents<'h, R> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, io::Error> {
        if buf.is_empty() {
            return Ok(0);
        }

        if self.extents.len() == self.current_extent {
            return Ok(0);
        }

        let current = &self.extents[self.current_extent];

        assert_lt!(self.current_pos, current.length);
        let to_read = usize((current.length - self.current_pos).min(u64(buf.len())));

        let actually_read = self.inner.read(&mut buf[..to_read])?;

        self.current_pos += u64(actually_read);

        if self.current_pos == current.length {
            self.current_extent += 1;

            if self.current_extent != self.extents.len() {
                let new_extent = &self.extents[self.current_extent];
                assert_eq!(
                    new_extent.index,
                    u64(self.current_extent),
                    "extent index out of sequence",
                );

                self.inner
                    .seek(SeekFrom::Start(self.base + new_extent.offset))?;
            }
        }

        Ok(actually_read)
    }
}

fn get_only_element<T, I: IntoIterator<Item = T>>(from: I) -> Result<T, Error> {
    let mut from = from.into_iter();
    let val = from.next().ok_or_else(|| format_err!("no items"))?;
    ensure!(from.next().is_none(), "unexpected second item");
    Ok(val)
}
