use std::collections::HashMap;
use std::ops::Deref;

use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};

/// scheme.json的结构。这个JSON文件将列出练习器实例所支持的所有方案。
/// 练习器开始加载时，这将会是练习器第一个下载的文件。
#[derive(Clone, Debug, Deserialize)]
pub struct Scheme {
    /// 方案ID，必须是独一无二的。
    pub id: String,
    /// 方案名字，面向用户。
    pub full_name: String,
    /// 方案字根集的URL，字根集的格式详情请参考 LoadedScheme 与 ZigenCategory 。
    /// 如果不是绝对地址，则默认根目录为 scheme.json 的所在目录。
    pub zigen_url: String,
    /// 方案字根集所需字体的URL。
    pub zigen_font: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct SchemeOptions {
    /// 乱序模式
    pub shuffle: bool,
    /// 简繁通练
    pub combined_training: bool,
    /// 繁体优先
    pub prioritize_trad: bool,
    /// 养老模式
    pub adept: bool,
}

/// 一个方案的字根集。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LoadedScheme<Z>(pub Vec<SchemeZigen<Z>>);

impl LoadedScheme<ZigenConfusableUnpopulated> {
    pub fn populate_confusables(self) -> LoadedScheme<ZigenConfusable> {
        let mut populated_confusables = self
            .0
            .iter()
            .filter_map(|zigen| match zigen {
                SchemeZigen::Confusable(con) => Some(con.zigens.as_slice()),
                _ => None,
            })
            .flatten()
            .map(|x| (x.clone(), None))
            .collect::<HashMap<_, _>>();

        self.0
            .iter()
            .filter_map(|zigen| match zigen {
                SchemeZigen::Category(cat) => Some(&cat.groups),
                _ => None,
            })
            .for_each(|groups| {
                for group in groups.iter() {
                    if populated_confusables.contains_key(&group.zigens[0]) {
                        *populated_confusables.get_mut(&group.zigens[0]).unwrap() =
                            Some(group.clone());
                    }
                }
            });

        let new_scheme =
            self.0
                .into_iter()
                .map(|zigen| match zigen {
                    SchemeZigen::Category(cat) => SchemeZigen::Category(cat),
                    SchemeZigen::Confusable(con) => SchemeZigen::Confusable({
                        let new_con = ZigenConfusable {
                            groups: con
                                .zigens
                                .iter()
                                .map(|z| {
                                    populated_confusables
                                        .remove(z)
                                        .unwrap()
                                        .expect(
                                            "混淆集使用的字根不在字根码表内，或不属于代表性字根",
                                        )
                                })
                                .collect(),
                            description: con.description.to_owned(),
                        };

                        new_con
                    }),
                })
                .collect::<Vec<_>>();

        LoadedScheme(new_scheme)
    }
}

impl LoadedScheme<ZigenConfusable> {
    pub fn sort_to_options(&mut self, options: &SchemeOptions) {
        let mut simps = Vec::new();
        let mut trads = Vec::new();
        let mut uncommons = Vec::new();

        let confusables = self
            .0
            .iter()
            .filter(|zigens| match zigens {
                SchemeZigen::Category(_) => false,
                SchemeZigen::Confusable(_) => true,
            })
            .cloned()
            .collect::<Vec<_>>();

        let categories = self.0.iter().filter_map(|zigens| match zigens {
            SchemeZigen::Category(cat) => Some((&cat.groups, &cat.description)),
            SchemeZigen::Confusable(_con) => None,
        });

        for (cat, cat_desc) in categories {
            let simp = ZigenCategory {
                groups: cat
                    .iter()
                    .filter(|group| {
                        (group.classify == ZigenClass::Simplified)
                            || (!options.prioritize_trad && group.classify == ZigenClass::Common)
                    })
                    .cloned()
                    .collect::<Vec<_>>(),
                description: cat_desc.to_owned(),
            };
            let trad = ZigenCategory {
                groups: cat
                    .iter()
                    .filter(|group| {
                        (group.classify == ZigenClass::Traditional)
                            || (options.prioritize_trad && group.classify == ZigenClass::Common)
                    })
                    .cloned()
                    .collect::<Vec<_>>(),
                description: cat_desc.to_owned(),
            };
            let uncommon = ZigenCategory {
                groups: cat
                    .iter()
                    .filter(|group| group.classify == ZigenClass::Uncommon)
                    .cloned()
                    .collect::<Vec<_>>(),
                description: cat_desc.to_owned(),
            };

            if !simp.groups.is_empty() {
                simps.push(SchemeZigen::Category(simp));
            }
            if !trad.groups.is_empty() {
                trads.push(SchemeZigen::Category(trad));
            }
            if !uncommon.groups.is_empty() {
                uncommons.push(SchemeZigen::Category(uncommon));
            }
        }

        if options.shuffle {
            trads.shuffle(&mut rand::rng());
            simps.shuffle(&mut rand::rng());
        }

        if options.prioritize_trad {
            if options.combined_training {
                trads.extend_from_slice(&simps);
            }
            trads.extend_from_slice(&uncommons);
            trads.extend_from_slice(&confusables);
            self.0 = trads;
        } else {
            if options.combined_training {
                simps.extend_from_slice(&trads);
            }
            simps.extend_from_slice(&uncommons);
            simps.extend_from_slice(&confusables);
            self.0 = simps;
        }
    }
}

/// 一个方案的字根集。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SchemeZigen<Z = ZigenConfusable> {
    /// 属于同一聚类，但编码不同的字根。
    #[serde(rename = "类")]
    Category(ZigenCategory),

    /// 容易被混淆或记错的几个字根。
    #[serde(rename = "混")]
    Confusable(Z),
}

impl SchemeZigen<ZigenConfusable> {
    pub fn as_raw_parts(&self) -> (&Vec<ZigenGroup>, &String) {
        let (zigen_groups, description) = match self {
            SchemeZigen::Category(cat) => (&cat.groups, &cat.description),

            SchemeZigen::Confusable(con) => (&con.groups, &con.description),
        };

        (zigen_groups, description)
    }

    pub fn as_raw_parts_mut(&mut self) -> (&mut Vec<ZigenGroup>, &mut String) {
        let (zigen_groups, description) = match self {
            SchemeZigen::Category(cat) => (&mut cat.groups, &mut cat.description),

            SchemeZigen::Confusable(con) => (&mut con.groups, &mut con.description),
        };

        (zigen_groups, description)
    }
}

/// 容易被混淆或记错的几个字根。
///
/// 这些字根未必属于同一个聚类，但是可能因为发音、外形、字源相似等因素，经常被
/// 学习者搞混。练习器会特意加强这类字根的学习强度。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZigenConfusable {
    groups: Vec<ZigenGroup>,
    #[serde(default)]
    description: String,
}

/// 容易被混淆或记错的几个字根。
///
/// 这些字根未必属于同一个聚类，但是可能因为发音、外形、字源相似等因素，经常被
/// 学习者搞混。练习器会特意加强这类字根的学习强度。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZigenConfusableUnpopulated {
    zigens: Vec<Zigen>,
    #[serde(default)]
    description: String,
}

/// 属于同一聚类的字根。这些字根的编码不一定相同（比如：宇浩星陈码中的Jm目和Jr日）。
///
/// 在双编码的输入法中，这些字根往往都会共享同一个大码。
///
/// 对那些实质上共同一个大码，但逻辑上不属于同一个聚类的字根（比如：宇浩星陈码中的Ug瓜和Ue业），
/// 不应该将其放入同一个 ZigenCategory 内，而应该将这些字根分成两个或者更多个 ZigenCategory 。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZigenCategory {
    pub groups: Vec<ZigenGroup>,
    #[serde(default)]
    pub description: String,
}

/// 位于同一聚类，外貌相似（或者根源相同），并且在归并后，编码一样的字根。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZigenGroup {
    pub zigens: Vec<Zigen>,
    pub code: String,
    pub classify: ZigenClass,
    pub description: String,
}

/// 单个字根。
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Zigen(pub String);

impl Deref for Zigen {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// 字根的分类。练习器会根据字根的分类与用户设置，推迟一些字根的出现时间。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZigenClass {
    /// 常用通用字根。
    #[serde(rename = "通")]
    Common,

    /// 常用简体字根。
    #[serde(rename = "简")]
    Simplified,

    /// 常用繁体字根。
    #[serde(rename = "繁")]
    Traditional,

    /// 不常用字根。
    #[serde(rename = "罕")]
    Uncommon,
}
