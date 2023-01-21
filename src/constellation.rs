pub struct Constellation
{
    pattern: Vec<u64>,
    offsets: Vec<u64>,
}



impl Constellation
{
    pub fn new(&self, pattern: Vec<u64>, offsets: Vec<u64>) -> Constellation
    {
        Constellation
        {
            pattern: pattern,
            offsets: offsets,
        }
    }

}