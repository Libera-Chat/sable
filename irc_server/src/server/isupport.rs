use std::cell::{
    Cell,
    Ref,
    RefCell
};

pub enum ISupportParam
{
    Simple,
    String(String),
    Int(i32)
}

pub struct ISupportEntry
{
    pub name: String,
    pub param: ISupportParam
}

impl ISupportEntry
{
    pub fn simple(name: &str) -> Self
    {
        Self { name: name.to_string(), param: ISupportParam::Simple }
    }

    pub fn string(name: &str, param: &str) -> Self
    {
        Self { name: name.to_string(), param: ISupportParam::String(param.to_string()) }
    }

    pub fn int(name: &str, param: i32) -> Self
    {
        Self { name: name.to_string(), param: ISupportParam::Int(param) }
    }

    fn format(&self) -> String
    {
        match &self.param
        {
            ISupportParam::Simple => self.name.clone(),
            ISupportParam::String(param) => format!("{}={}", self.name, param),
            ISupportParam::Int(param) => format!("{}={}", self.name, param),
        }
    }
}

pub struct ISupportBuilder
{
    entries: Vec<ISupportEntry>,
    cache: RefCell<Option<Vec<String>>>
}

impl ISupportBuilder
{
    pub fn new() -> Self
    {
        Self {
            entries: Vec::new(),
            cache: RefCell::new(None)
        }
    }

    pub fn add(&mut self, entry: ISupportEntry)
    {
        self.entries.push(entry);
        self.cache.replace(None);
    }

    pub fn data(&self) -> Ref<Vec<String>>
    {
        if self.cache.borrow().is_none()
        {
            self.build()
        }
        Ref::map(self.cache.borrow(),
            |r| r.as_ref().expect("Failed to build isupport cache")
        )
    }

    fn build(&self)
    {
        const MAX_LEN:usize = 300;

        let mut result = Vec::new();
        let mut current = Cell::new(String::new());

        for (i, entry) in (&self.entries).iter().enumerate()
        {
            let s = entry.format();

            if current.get_mut().len() + s.len() + 1 > MAX_LEN
            {
                result.push(current.replace(String::new()));
            }
            else if i > 0
            {
                // if we're not making a new line and we're not the first
                // item of the first line, we need a space
                current.get_mut().push(' ');
            }
            current.get_mut().push_str(&s);
        }

        result.push(current.replace(String::new()));

        self.cache.replace(Some(result));
    }
}