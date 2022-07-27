mod conjugations;
use conjugations::VerbConjugations;

fn main() {
    let verb_conjugations = VerbConjugations::get_conjugation_tables();
    for table in verb_conjugations.conjugation_tables {
        println!("{} {:?}", table.tense, table.conjugations);
    }
}
