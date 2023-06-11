pub trait OperationHandler {
    fn run(&mut self) -> Result<(), &'static str>;
}
