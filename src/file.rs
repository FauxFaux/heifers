use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::collections::HashSet;
use std::io::Read;

use cast::u32;
use cast::usize;
use failure::Error;

use mpeg;
use mpeg::iprp::Property;
use mpeg::FourCc;
use mpeg::ItemInfo;

// TODO: not Debug
#[derive(Debug)]
pub struct Heif {
    handler: FourCc,
    primary_item: u32,
    items: HashMap<u32, ItemInfo>,
    props: Vec<(HashSet<u32>, Property)>,
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
            for item_info in item_infos {
                match items.entry(u32(item_info.id)) {
                    Entry::Occupied(_) => bail!("duplicate item id: {}", item_info.id),
                    Entry::Vacant(vacancy) => vacancy.insert(item_info),
                };
            }
        }

        Ok(Heif {
            handler,
            primary_item,
            items,
            props,
        })
    }
}

fn get_only_element<T, I: IntoIterator<Item = T>>(from: I) -> Result<T, Error> {
    let mut from = from.into_iter();
    let val = from.next().ok_or_else(|| format_err!("no items"))?;
    ensure!(from.next().is_none(), "unexpected second item");
    Ok(val)
}
