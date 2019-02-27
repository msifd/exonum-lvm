pub trait CurrencyApi {
    fn transfer(receiver: &String, amount: u64);
}