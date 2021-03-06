use crate::yarp_data::*;
use heck::*;
use idcontain::{Id, IdSlab};
use indexmap::IndexMap;
use itertools::Itertools;
use liquid::value::liquid_value;
use liquid::value::map::Map as LiquidMap;
use liquid::value::Value as LiquidValue;
use std::mem::replace;

#[derive(Debug, Default)]
pub struct IdRegistry {
    slab: IdSlab<UnitIdentifier>,
    // by_uid: IndexMap<String, Id<UnitIdentifier>>,
    // by_rawid: IndexMap<String, Id<UnitIdentifier>>,
}

impl IdRegistry {
    fn insert(&mut self, id: UnitIdentifier) -> UnitIdentifier {
        let id = self.slab.insert(id);

        // self.by_uid.insert(str_uid, id);
        // self.by_rawid.insert(str_rawid, id);

        self.slab[id].clone()
    }

    // fn get_by_uid<'a, 'b>(&'a self, uid: &'b str) -> &'a UnitIdentifier {
    //     &self.slab[self.by_uid[uid]]
    // }

    // fn get_by_rawid<'a, 'b>(&'a self, constant_name: &'b str) -> &'a UnitIdentifier {
    //     &self.slab[self.by_rawid[constant_name]]
    // }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum UnitIdentifier {
    UID { uid: String, constant_name: String },
    RawID { rawid: String },
}

impl UnitIdentifier {
    fn new_custom(uid: String) -> UnitIdentifier {
        UnitIdentifier::UID {
            constant_name: uid.to_shouty_snake_case(),
            uid,
        }
    }

    fn new_stock(rawid: String) -> UnitIdentifier {
        UnitIdentifier::RawID { rawid }
    }

    pub fn is_uid(&self) -> bool {
        if let UnitIdentifier::UID { .. } = &self {
            true
        } else {
            false
        }
    }

    pub fn is_rawid(&self) -> bool {
        if let UnitIdentifier::RawID { .. } = &self {
            true
        } else {
            false
        }
    }

    pub fn uid(&self) -> &str {
        if let UnitIdentifier::UID { uid, .. } = &self {
            &uid
        } else {
            panic!("cannot call .uid() on non-UID variant")
        }
    }

    pub fn constant(&self) -> String {
        match self {
            UnitIdentifier::UID { constant_name, .. } => constant_name.to_string(),
            UnitIdentifier::RawID { rawid, .. } => format!("'{}'", rawid)
        }
    }

    pub fn rawid(&self) -> &str {
        if let UnitIdentifier::RawID { rawid } = &self {
            &rawid
        } else {
            panic!("cannot call .rawid() on non-RawID variant")
        }
    }
}

#[derive(Debug, Default)]
pub struct UnitRegistry {
    pub registry: IndexMap<UnitIdentifier, YarpUnit>,
}

impl UnitRegistry {
    fn insert(&mut self, unit: YarpUnit) {
        self.registry.insert(unit.id().clone(), unit);
    }

    fn get_mut(&mut self, id: &UnitIdentifier) -> &mut YarpUnit {
        self.registry.get_mut(id).unwrap()
    }

    pub fn get(&self, id: &UnitIdentifier) -> &YarpUnit {
        &self.registry[id]
    }
}

#[derive(Debug)]
pub enum YarpUnitVariant {
    Unit,
    Building,
    UnitShop {
        sold_ids: Vec<UnitIdentifier>,
        scale: f32,
    },
    Builder {
        built_ids: Vec<UnitIdentifier>,
    },
}

#[derive(Debug)]
pub enum YarpUnit {
    Custom {
        id: UnitIdentifier,
        variant: YarpUnitVariant,
        name: String,
        model: String,
        icon: String,
    },
    Stock {
        id: UnitIdentifier,
        model: String,
    },
}

impl YarpUnit {
    fn new_unit(id: UnitIdentifier, name: String, model: String, icon: String) -> YarpUnit {
        YarpUnit::Custom {
            id,
            name,
            icon,
            model: model.trim().to_string(),
            variant: YarpUnitVariant::Unit,
        }
    }

    fn new_building(id: UnitIdentifier, name: String, model: String, icon: String) -> YarpUnit {
        YarpUnit::Custom {
            id,
            name,
            icon,
            model: model.trim().to_string(),
            variant: YarpUnitVariant::Building,
        }
    }

    fn new_shop(
        id: UnitIdentifier,
        name: String,
        model: String,
        sold_ids: &[UnitIdentifier],
        scale: f32,
    ) -> YarpUnit {
        YarpUnit::Custom {
            id,
            name,
            icon: "".to_string(),
            model: model.trim().to_string(),
            variant: YarpUnitVariant::UnitShop {
                sold_ids: sold_ids.into(),
                scale,
            },
        }
    }

    fn new_builder(
        id: UnitIdentifier,
        name: String,
        model: String,
        icon: String,
        built_ids: &[UnitIdentifier],
    ) -> YarpUnit {
        YarpUnit::Custom {
            id,
            name,
            icon,
            model: model.trim().to_string(),
            variant: YarpUnitVariant::Builder {
                built_ids: built_ids.into(),
            },
        }
    }

    fn new_with_variant(
        id: UnitIdentifier,
        name: String,
        model: String,
        icon: String,
        variant: YarpUnitVariant,
    ) -> YarpUnit {
        YarpUnit::Custom {
            id,
            name,
            icon,
            model: model.trim().to_string(),
            variant,
        }
    }

    fn new_stock(id: UnitIdentifier, model: String) -> YarpUnit {
        YarpUnit::Stock { id, model }
    }

    pub fn id(&self) -> &UnitIdentifier {
        match &self {
            YarpUnit::Custom { id, .. } => id,
            YarpUnit::Stock { id, .. } => id,
        }
    }

    pub fn model(&self) -> &str {
        match &self {
            YarpUnit::Custom { model, .. } => model,
            YarpUnit::Stock { model, .. } => model,
        }
    }

    pub fn liquid_value(&self) -> LiquidValue {
        match &self {
            YarpUnit::Custom {
                id,
                variant,
                name,
                model,
                icon,
            } => {
                let mut value = LiquidMap::new();
                value.insert(
                    "constant".into(),
                    LiquidValue::scalar(id.constant().to_string()),
                );
                value.insert("model".into(), LiquidValue::scalar(model.to_string()));
                value.insert("name".into(), LiquidValue::scalar(name.to_string()));
                value.insert("icon".into(), LiquidValue::scalar(icon.to_string()));

                match variant {
                    YarpUnitVariant::Builder { built_ids } => {
                        value.insert(
                            "built".into(),
                            LiquidValue::scalar(
                                built_ids
                                    .iter()
                                    .map(|s| format!("{}.toRawCode()", s.constant()))
                                    .join(" + \",\" + "),
                            ),
                        );
                    }
                    YarpUnitVariant::UnitShop { sold_ids, .. } => {
                        value.insert(
                            "sold".into(),
                            LiquidValue::scalar(
                                sold_ids
                                    .iter()
                                    .map(|s| {
                                        if s.is_rawid() {
                                            format!("\"{}\"", s.rawid())
                                        } else {
                                            format!("{}.toRawCode()", s.constant())
                                        }
                                    })
                                    .join(" + \",\" + "),
                            ),
                        );
                    }
                    _ => {}
                }

                LiquidValue::Object(value)
            }
            _ => LiquidValue::nil(),
        }
    }

    pub fn liquid_insert_into_context(&self, context: &mut LiquidMap) {
        if let YarpUnit::Custom { variant, .. } = &self {
            let value = self.liquid_value();

            match variant {
                YarpUnitVariant::Unit => context
                    .get_mut("units")
                    .unwrap()
                    .as_array_mut()
                    .unwrap()
                    .push(value),
                YarpUnitVariant::Building => context
                    .get_mut("buildings")
                    .unwrap()
                    .as_array_mut()
                    .unwrap()
                    .push(value),
                YarpUnitVariant::Builder { .. } => context
                    .get_mut("builders")
                    .unwrap()
                    .as_array_mut()
                    .unwrap()
                    .push(value),
                YarpUnitVariant::UnitShop { .. } => context
                    .get_mut("shops")
                    .unwrap()
                    .as_array_mut()
                    .unwrap()
                    .push(value),
            }
        }
    }
}

#[derive(Default)]
pub struct ModelRegistry {
    pub registry: IndexMap<UnitIdentifier, String>,
}

impl ModelRegistry {
    fn insert(&mut self, id: &UnitIdentifier, model: String) {
        self.registry.insert(id.clone(), model);
    }
}

#[derive(Default)]
pub struct Registries {
    pub id: IdRegistry,
    pub unit: UnitRegistry,
    pub model: ModelRegistry,
}



fn transform_yarp_data_unit(unit: &YarpDataUnit, registries: &mut Registries) -> UnitIdentifier {
    match unit {
        YarpDataUnit::Custom(custom_unit) => {
            let variant = match &custom_unit.variant {
                YarpDataUnitVariant::Unit => YarpUnitVariant::Unit,
                YarpDataUnitVariant::Building => YarpUnitVariant::Building,
                YarpDataUnitVariant::Builder { built } => YarpUnitVariant::Builder {
                    built_ids: built
                        .iter()
                        .map(|s| transform_yarp_data_unit(s, registries))
                        .collect(),
                },
            };

            let id = registries
                .id
                .insert(UnitIdentifier::new_custom(custom_unit.uid.to_string()));

            let yarp_unit = YarpUnit::new_with_variant(
                id.clone(),
                custom_unit.name.to_string(),
                custom_unit.model.to_string(),
                custom_unit.icon.to_string(),
                variant,
            );

            registries.unit.insert(yarp_unit);

            id
        }
        YarpDataUnit::Stock(stock_unit) => {
            let id = registries
                .id
                .insert(UnitIdentifier::new_stock(stock_unit.rawid.to_string()));
            let yarp_unit = YarpUnit::new_stock(id.clone(), stock_unit.model.to_string());
            registries.unit.insert(yarp_unit);

            id
        }
    }
}

pub fn transform_yarp_data(data: &YarpData) -> Registries {
    let mut registries = Registries::default();

    for unit_shop in data.shops.iter().flat_map(|(_, s)| s.iter()) {
        let mut sold_ids: Vec<UnitIdentifier> = Vec::new();

        for unit in unit_shop.sold.iter() {
            sold_ids.push(transform_yarp_data_unit(unit, &mut registries));
        }

        let id = registries
            .id
            .insert(UnitIdentifier::new_custom(unit_shop.uid.to_string()));
        let yarp_unit = YarpUnit::new_shop(
            id,
            unit_shop.name.to_string(),
            unit_shop.model.to_string(),
            &sold_ids,
            unit_shop.scale,
        );
        registries.unit.insert(yarp_unit);
    }

    for (rawid, model) in data.stock_model_registry.iter() {
        let id = registries
            .id
            .insert(UnitIdentifier::new_stock(rawid.to_string()));
        registries.model.insert(&id, model.to_string());
    }

    registries
}

pub fn liquid_context(registries: &Registries) -> LiquidValue {
    let mut value =
        liquid_value!({"units" : [], "buildings" : [], "shops" : [], "builders" : [], "uids" : [], "models" : []});

    for (_, unit) in &registries.unit.registry {
        unit.liquid_insert_into_context(value.as_object_mut().unwrap());
    }

    for id in &registries.id.slab {
        if let UnitIdentifier::UID { .. } = &id {
            value
                .as_object_mut()
                .unwrap()
                .get_mut("uids")
                .unwrap()
                .as_array_mut()
                .unwrap()
                .push(liquid_value!({
                    "constant" : id.constant(),
                    "uid" : id.uid(),
                }));
        }
    }

    for (id, model) in &registries.model.registry {
        value
            .as_object_mut()
            .unwrap()
            .get_mut("models")
            .unwrap()
            .as_array_mut()
            .unwrap()
            .push(liquid_value!({
                "constant" : id.constant(),
                "path" : model.to_string(),
            }));
    }

    value
}
