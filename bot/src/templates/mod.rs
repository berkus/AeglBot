pub trait ToTemplate {
    type Template;
    fn to_template(&self) -> Self::Template;
}
