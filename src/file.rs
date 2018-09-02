use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::collections::HashSet;
use std::io::Read;

use cast::u32;
use cast::usize;
use failure::Error;

use mpeg;

// TODO: not Debug
#[derive(Debug)]
pub struct Heif {}

impl Heif {
    pub fn new<R: Read>(from: R) -> Result<Heif, Error> {
        let raw = mpeg::load_meta(from)?;

        let handler = get_only_element(&raw.handler)?;
        let primary_item = get_only_element(&raw.primary_item)?;

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

                props.push((prop, associated_items));
            }
        }

        for item_infos in raw.item_infos {
            for item_info in item_infos {
                let id = u32(item_info.id);
            }
        }

        Ok(Heif {})
    }
}

fn get_only_element<T, I: IntoIterator<Item = T>>(from: I) -> Result<T, Error> {
    let mut from = from.into_iter();
    let val = from.next().ok_or_else(|| format_err!("no items"))?;
    ensure!(from.next().is_none(), "unexpected second item");
    Ok(val)
}
