#![feature(box_patterns)]

use syn::visit::Visit;
use syn::*;

pub struct FindImplementorsVisiter {
    structs: Vec<ItemStruct>,
    impls: Vec<ItemImpl>,
    ident: Option<Ident>,
    found: Vec<Struct>,
    search: Path,
}

pub struct Struct {
    pub struct_name: String,
    pub struct_doc: Option<String>,
    pub implements: String,
}

impl FindImplementorsVisiter {
    fn new(search: Path) -> Self {
        FindImplementorsVisiter {
            structs: Vec::new(),
            impls: Vec::new(),
            ident: None,
            found: Vec::new(),
            search,
        }
    }

    pub fn file(source: &str, trait_: &str) -> Result<Vec<Struct>> {
        let trait_path = syn::parse_str::<syn::Path>(trait_)?;
        let parsed = syn::parse_file(source)?;
        let mut visitor = Self::new(trait_path);

        visitor.visit_file(&parsed);

        visitor.find_implementors();

        Ok(visitor.found)
    }

    fn find_implementors(&mut self) {
        for struct_ in &self.structs {
            // check if struct implements System
            for impl_ in &self.impls {
                if let box Type::Path(x) = &impl_.self_ty {
                    let path = &x.path;
                    if path.segments[path.segments.len() - 1].ident == struct_.ident {
                        let doc = struct_doc(struct_);
                        let doc = if doc == "" { None } else { Some(doc) };
                        let struct_ = Struct {
                            struct_name: struct_.ident.to_string(),
                            struct_doc: doc,
                            implements: path_to_string(&x.path),
                        };
                        self.found.push(struct_);
                    }
                }
            }
        }
    }
}

impl<'ast> Visit<'ast> for FindImplementorsVisiter {
    fn visit_item_impl(&mut self, i: &'ast ItemImpl) {
        visit::visit_item_impl(self, i);
        if let Some(trait_) = &i.trait_ {
            let path = &trait_.1;
            if path.segments[path.segments.len() - 1].ident
                == self.search.segments[self.search.segments.len() - 1].ident
            {
                self.impls.push(i.clone());
            }
        }
    }

    fn visit_item_struct(&mut self, i: &'ast ItemStruct) {
        visit::visit_item_struct(self, i);
        self.ident = Some(i.ident.clone());
        self.structs.push(i.clone());
    }
}

fn struct_doc(struct_: &ItemStruct) -> String {
    let mut string = String::new();
    for attr in &struct_.attrs {
        if let Some(x) = extract_documentation(&attr) {
            string.push_str(&x);
            string.push('\n');
        }
    }
    // drop last \n
    string.pop();
    string
}

fn path_to_string(path: &Path) -> String {
    let mut string = String::new();
    if let Some(_colons) = &path.leading_colon {
        string.push_str("::");
    }

    for seg in path.segments.pairs() {
        string.push_str(&seg.value().ident.to_string());
        if let Some(_) = seg.punct() {
            string.push_str("::");
        }
    }

    string
}

fn extract_documentation(attr: &Attribute) -> Option<String> {
    if path_to_string(&attr.path) == "doc" {
        for token in attr.tts.clone().into_iter() {
            match token {
                proc_macro2::TokenTree::Literal(lit) => {
                    let mut doc_string = lit.to_string();
                    // remove "" and leading whitespace
                    doc_string = doc_string[1..doc_string.len() - 1].trim_left().to_string();
                    return Some(doc_string);
                }
                _ => {}
            }
        }
        None
    } else {
        None
    }
}
