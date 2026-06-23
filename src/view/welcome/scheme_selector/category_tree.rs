use crate::scheme::Scheme;

pub(super) type CategoryIter<'a> = std::slice::Iter<'a, CategoryNode>;

/// `CategoryTree`只是一个简单的工具类型，功能是将schemes.json里的category转换成一个树状结构
#[derive(Clone, Debug, PartialEq)]
pub(super) struct CategoryTree {
    root: CategoryNode,
}

impl CategoryTree {
    pub fn new(schemes: &Vec<Scheme>) -> Self {
        // 根节点其实就只是一个无名的普通节点
        let mut root = CategoryNode::new();
        for scheme in schemes {
            root.insert(&scheme.category);
        }

        CategoryTree { root }
    }

    pub fn children<'a>(&'a self) -> CategoryIter<'a> {
        self.root.children()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct CategoryNode {
    name: String,
    children: Vec<Self>,
}

impl CategoryNode {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            children: Vec::new(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    fn insert(&mut self, category: &[String]) {
        if category.len() == 0 {
            return;
        }

        let (first, rest) = category.split_first().unwrap();

        for child in &mut self.children {
            if *first == child.name {
                return child.insert(rest);
            }
        }

        let mut new_child = Self {
            name: first.to_owned(),
            children: Vec::new(),
        };
        new_child.insert(rest);
        self.children.push(new_child);
    }

    pub fn children<'a>(&'a self) -> CategoryIter<'a> {
        self.children.iter()
    }
}
