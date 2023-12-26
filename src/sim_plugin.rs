#![allow(unused)]

use bevy::{ecs::schedule::ScheduleLabel, prelude::*};
use rand::Rng;
use textplots::{utils, AxisBuilder, Chart, Plot, Shape};

//markers
#[derive(Component)]
struct Person;

#[derive(Component)]
struct Property;

#[derive(Component)]
struct GroceryStore;

//components

#[derive(Component)]
struct Age(u32);

#[derive(Component)]
struct Rent {
    amount: f32,
}

#[derive(Component)]
struct OwnedBy {
    owner: Entity,
}
#[derive(Component)]
struct LivesIn {
    property: Entity,
}

#[derive(Component)]
struct BankAccount {
    balance: f32,
}

#[derive(Component)]
struct Job {
    salary: f32,
}

#[derive(Component)]
struct Product {
    price: f32,
}

#[derive(Bundle)]
struct PersonBundle {
    person: Person,
    bank_account: BankAccount,
    job: Job,
    age: Age,
}

//bundles
#[derive(Bundle)]
struct LandlordBundle {
    person: Person,
    bank_account: BankAccount,
    lives_in: LivesIn,
    job: Job,
}

#[derive(Bundle)]
struct TenantBundle {
    person: Person,
    bank_account: BankAccount,
    lives_in: LivesIn,
    job: Job,
}

#[derive(Bundle)]
struct PropertyBundle {
    property: Property,
    rent: Rent,
    owned_by: OwnedBy,
}

#[derive(Bundle)]
struct GroceryStoreBundle {
    grocery_store: GroceryStore,
    product: Product,
}

fn populate_landlord_world(mut commands: Commands) {
    let number_of_renters = 1000;
    let number_of_home_owners = 100;

    let place_holder_entity = commands.spawn(()).id();

    let mut landlords = Vec::new();

    for _ in 0..number_of_home_owners {
        let property_land_lord_lives_in = commands
            .spawn(PropertyBundle {
                property: Property,
                rent: Rent { amount: 1000.0 },
                owned_by: OwnedBy {
                    owner: place_holder_entity,
                },
            })
            .id();

        let landlord = commands
            .spawn(LandlordBundle {
                person: Person,
                bank_account: BankAccount { balance: 1000.0 },
                lives_in: LivesIn {
                    property: property_land_lord_lives_in,
                },
                job: Job { salary: 1000.0 },
            })
            .id();

        // fix the property to have the landlord as the owner
        commands
            .entity(property_land_lord_lives_in)
            .insert(OwnedBy { owner: landlord });

        landlords.push(landlord);
    }

    // create the properties the landlord owns
    // run through the landlords giving them a house
    // until all the houses are owned
    let mut upto_landlord_index = 0;
    for _ in 0..number_of_renters {
        let property = commands
            .spawn(PropertyBundle {
                property: Property,
                rent: Rent { amount: 1000.0 },
                owned_by: OwnedBy {
                    owner: place_holder_entity,
                },
            })
            .id();

        // fix the property to have the landlord as the owner
        let landlord = landlords[upto_landlord_index];
        commands
            .entity(property)
            .insert(OwnedBy { owner: landlord });

        // create the tenant
        commands.spawn(TenantBundle {
            person: Person,
            bank_account: BankAccount { balance: 500.0 },
            lives_in: LivesIn { property: property },
            job: Job { salary: 1000.0 },
        });

        //increment to next landlord
        upto_landlord_index = (upto_landlord_index + 1) % landlords.len();
    }
}

fn populate_person_world(mut commands: Commands) {
    let number_of_people = 100;
    let min_income = 10_000.0;
    let max_income = 100_000.0;
    let min_wealth = 10_000.0;
    let max_wealth = 1_000_000.0;
    let max_age = 80;
    let rng = &mut rand::thread_rng();

    for _ in 0..number_of_people {
        let income = rng.gen_range(min_income..max_income);
        let wealth = rng.gen_range(min_wealth..max_wealth);
        let age = rng.gen_range(0..max_age);
        commands.spawn(PersonBundle {
            person: Person,
            bank_account: BankAccount { balance: wealth },
            job: Job { salary: income },
            age: Age(0),
        });
    }

    //add grocery store
    commands.spawn(GroceryStoreBundle {
        grocery_store: GroceryStore,
        product: Product { price: 100.0 },
    });
}

fn print_bank_accounts_system(
    landlords: Query<(&BankAccount), Without<Rent>>,
    renters: Query<(&BankAccount), With<Rent>>,
) {
    // average landlords bank account
    let landlord_bank_account: f32 =
        landlords.iter().fold(0.0, |acc, b| b.balance + acc) / landlords.iter().len() as f32;
    let renter_bank_account: f32 =
        renters.iter().fold(0.0, |acc, b| b.balance + acc) / landlords.iter().len() as f32;

    println!("Landlord bank account: {}", landlord_bank_account);
    println!("Renter bank account: {}", renter_bank_account);
}

fn food_cost_system(mut people: Query<(&mut BankAccount, &Job)>) {
    let rng = &mut rand::thread_rng();

    let food_cost = 10_000.0;
    for (mut bank_account, job) in people.iter_mut() {
        bank_account.balance -= food_cost;
    }
}

fn product_inflation_system(mut products: Query<(&mut Product)>) {
    let inflation_rate = 0.05;
    for (mut product) in products.iter_mut() {
        product.price += product.price * inflation_rate;
    }
}

fn kill_if_poor_system(mut commands: Commands, mut people: Query<(Entity, &BankAccount)>) {
    for (entity, bank_account) in people.iter_mut() {
        if bank_account.balance < 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

fn plot_wealth_distribution_instantaneous(bank_acounts: Query<(&BankAccount)>) {
    let total_wealth = bank_acounts.iter().fold(0.0, |acc, b| b.balance + acc);
    let total_number_of_people = bank_acounts.iter().len() as f32;
    let data = bank_acounts
        .iter()
        .map(|b| (0.0, b.balance))
        .collect::<Vec<_>>();
    let max_val = bank_acounts
        .iter()
        .map(|b| b.balance)
        .fold(0.0, |acc, b| if b > acc { b } else { acc });
    // make max closest power of 10
    let hist = utils::histogram(&data, -0.1, max_val, 16);
    let hist = hist
        .iter()
        .map(|(a, b)| (*a, *b / total_number_of_people))
        .collect::<Vec<_>>();
    // let max_val = max_val / total_wealth;

    println!("Wealth distribution of last people");
    Chart::new(180, 30, 0.0, max_val)
        .lineplot(&Shape::Bars(hist.as_slice()))
        .nice();
}

#[derive(Resource)]
struct IterationCount(usize);

#[derive(Resource)]
struct PopulationStatistics {
    number_of_people: Vec<u32>,
    average_age: Vec<u32>,
    average_wealth: Vec<f32>,
    gini_coefficient: Vec<f32>,
    max_wealth: Vec<f32>,
}

impl PopulationStatistics {
    fn new() -> Self {
        Self {
            number_of_people: Vec::new(),
            average_age: Vec::new(),
            average_wealth: Vec::new(),
            gini_coefficient: Vec::new(),
            max_wealth: Vec::new(),
        }
    }
}

fn update_iteration_count(mut iteration_count: ResMut<IterationCount>) {
    iteration_count.0 += 1;
}

fn subsidise_the_poor(mut people: Query<(&mut BankAccount)>) {
    for (mut bank_account) in people.iter_mut() {
        if bank_account.balance < 0.0 {
            bank_account.balance = 0.0;
        }
    }
}

#[derive(Resource)]
struct GreetTimer(Timer);

const ITERATIONS: usize = 200;

fn runner(mut app: App) {
    for _ in 0..ITERATIONS {
        app.update();
    }
}

fn is_at_start(iteration_count: Res<IterationCount>) -> bool {
    iteration_count.0 == 0
}

fn is_at_end(iteration_count: Res<IterationCount>) -> bool {
    iteration_count.0 + 1 == ITERATIONS
}

fn age_people(mut people: Query<(&mut Age)>) {
    for (mut age) in people.iter_mut() {
        age.0 += 1;
    }
}

#[derive(Event)]
struct DeathEvent(Entity);
#[derive(Event)]
struct BornEvent(Entity);

fn people_die(
    mut commands: Commands,
    people: Query<(Entity, &Age), With<Person>>,
    mut ev_death: EventWriter<DeathEvent>,
) {
    for (entity, age) in people.iter() {
        if age.0 > 80 {
            ev_death.send(DeathEvent(entity));
        }
    }
}

fn despawn_dead_people(mut commands: Commands, mut ev_death: EventReader<DeathEvent>) {
    for death_event in ev_death.read() {
        if commands.get_entity(death_event.0).is_some() {
            commands.entity(death_event.0).despawn();
        }
    }
}

fn people_born(
    mut commands: Commands,
    mut ev_death: EventReader<DeathEvent>,
    mut pop_statistics: Res<PopulationStatistics>,
    people: Query<(&BankAccount)>,
) {
    let rng = &mut rand::thread_rng();
    let max_wealth = pop_statistics.max_wealth.last().unwrap().max(10.0);
    let max_number_of_kids = 2;

    for death_event in ev_death.read() {
        let bank_account = people.get(death_event.0).unwrap();
        let wealth = bank_account.balance.max(10.0);

        let number_of_children = {
            // number of children is based of the amount of wealth
            // richer people have more children
            let max_wealth_log = f32::log10(max_wealth);
            let wealth_log = f32::log10(wealth);
            let wealth_ratio = wealth_log / max_wealth_log;
            // println!("Max wealth: {}", max_wealth);
            // println!("Wealth: {}", wealth);
            // println!("Max wealth log: {}", max_wealth_log);
            // println!("Wealth log: {}", wealth_log);
            // println!("Wealth ratio: {}", wealth_ratio);
            let number_of_children_fraction = (wealth_ratio * max_number_of_kids as f32);

            number_of_children_fraction.round() as u32
        };

        // use random number to decide if a child is born
        // println!("Number of children: {}", number_of_children);
        // let number_of_children = (6 - number_of_children as i32).max(2).min(1);
        for _ in 0..number_of_children {
            let income = rng.gen_range(10_000.0..100_000.0);
            let childs_wealth = wealth / (number_of_children as f32);
            commands.spawn(PersonBundle {
                person: Person,
                bank_account: BankAccount {
                    balance: childs_wealth,
                },
                job: Job { salary: income },
                age: Age(0),
            });
        }
    }
}

const RETURN_RATE: f32 = 0.08;

fn bank_investment_returns(mut people: Query<(&mut BankAccount)>) {
    let rng = &mut rand::thread_rng();

    for (mut bank_account) in people.iter_mut() {
        bank_account.balance += bank_account.balance * RETURN_RATE;
    }
}

fn record_number_of_people(
    mut population_statistics: ResMut<PopulationStatistics>,
    people: Query<(&Person)>,
) {
    population_statistics
        .number_of_people
        .push(people.iter().len() as u32);
}

fn record_max_wealth(
    mut population_statistics: ResMut<PopulationStatistics>,
    people: Query<(&BankAccount)>,
) {
    let max_wealth = people.iter().fold(0.0, |acc, (bank_account)| {
        if bank_account.balance > acc {
            bank_account.balance
        } else {
            acc
        }
    });
    population_statistics.max_wealth.push(max_wealth);
}

fn record_age_distribution(
    mut population_statistics: ResMut<PopulationStatistics>,
    people: Query<(&Age)>,
) {
    let average_age = {
        if people.iter().len() == 0 {
            0
        } else {
            people.iter().fold(0, |acc, age| acc + age.0) / people.iter().len() as u32
        }
    };
    population_statistics.average_age.push(average_age);
}

fn record_wealth_distribution(
    mut population_statistics: ResMut<PopulationStatistics>,
    people: Query<(&BankAccount)>,
) {
    let average_wealth = people
        .iter()
        .fold(0.0, |acc, (bank_account)| acc + bank_account.balance)
        / people.iter().len() as f32;
    population_statistics.average_wealth.push(average_wealth);
}

fn record_gini_coefficient(
    mut population_statistics: ResMut<PopulationStatistics>,
    people: Query<(&BankAccount)>,
) {
    // https://en.wikipedia.org/wiki/Gini_coefficient#Alternative_expressions

    let n = people.iter().len() as f32;
    let sum = people
        .iter()
        .fold(0.0, |acc, (bank_account)| acc + bank_account.balance);

    let mut sum_diff = 0.0;

    for (bank_account) in people.iter() {
        for (bank_account_2) in people.iter() {
            sum_diff += (bank_account_2.balance - bank_account.balance).abs();
        }
    }

    let g = sum_diff / (2.0 * n * sum);

    population_statistics.gini_coefficient.push(g);
}

fn plot_age_distribution(mut population_statistics: ResMut<PopulationStatistics>) {
    let data = population_statistics
        .number_of_people
        .iter()
        .enumerate()
        .map(|(i, &n)| (i as f32, n as f32))
        .collect::<Vec<_>>();
    // create a line chart
    println!("Age distribution");
    Chart::new(180, 30, 0.0, ITERATIONS as f32)
        .lineplot(&Shape::Lines(&data))
        .nice();
}

fn plot_wealth_distribution(mut population_statistics: ResMut<PopulationStatistics>) {
    let data = population_statistics
        .average_wealth
        .iter()
        .enumerate()
        .map(|(i, &n)| (i as f32, n as f32))
        .collect::<Vec<_>>();
    // create a line chart
    println!("Wealth distribution");
    Chart::new(180, 30, 0.0, ITERATIONS as f32)
        .lineplot(&Shape::Lines(&data))
        .nice();
}

fn plot_max_wealth(mut population_statistics: ResMut<PopulationStatistics>) {
    let data = population_statistics
        .max_wealth
        .iter()
        .enumerate()
        .map(|(i, &n)| (i as f32, n as f32))
        .collect::<Vec<_>>();
    // create a line chart
    println!("Max wealth");
    Chart::new(180, 30, 0.0, ITERATIONS as f32)
        .lineplot(&Shape::Lines(&data))
        .nice();
}

fn plot_population_size(mut population_statistics: ResMut<PopulationStatistics>) {
    let data = population_statistics
        .number_of_people
        .iter()
        .enumerate()
        .map(|(i, &n)| (i as f32, n as f32))
        .collect::<Vec<_>>();
    // create a line chart
    println!("Population size");
    Chart::new(180, 30, 0.0, ITERATIONS as f32)
        .lineplot(&Shape::Lines(&data))
        .nice();
}

fn plot_gini_coefficient(mut population_statistics: ResMut<PopulationStatistics>) {
    let data = population_statistics
        .gini_coefficient
        .iter()
        .enumerate()
        .map(|(i, &n)| (i as f32, n as f32))
        .collect::<Vec<_>>();
    // create a line chart
    println!("Gini coefficient");
    Chart::new(180, 30, 0.0, ITERATIONS as f32)
        .lineplot(&Shape::Lines(&data))
        .nice();
}

#[derive(ScheduleLabel, Debug, Hash, PartialEq, Eq, Clone)]
struct UpdateData;

#[derive(ScheduleLabel, Debug, Hash, PartialEq, Eq, Clone)]
struct AddEntities;

#[derive(ScheduleLabel, Debug, Hash, PartialEq, Eq, Clone)]
struct DeleteEntities;

#[derive(ScheduleLabel, Debug, Hash, PartialEq, Eq, Clone)]
struct RecordData;

fn run_custom_update_schedule(world: &mut World) {
    world.run_schedule(RecordData);
    world.run_schedule(UpdateData);
    world.run_schedule(DeleteEntities);
    world.run_schedule(AddEntities);
}

pub struct BreakupUpdateSchedulePlugin;

impl Plugin for BreakupUpdateSchedulePlugin {
    fn build(&self, app: &mut App) {
        let update_data_schedule: Schedule = Schedule::new(UpdateData);
        let add_entities_schedule: Schedule = Schedule::new(AddEntities);
        let delete_entities_schedule: Schedule = Schedule::new(DeleteEntities);

        app.add_schedule(update_data_schedule)
            .add_schedule(add_entities_schedule)
            .add_schedule(delete_entities_schedule)
            .add_systems(Update, run_custom_update_schedule);
    }
}

pub struct SimPlugin;

impl Plugin for SimPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<DeathEvent>()
            .set_runner(runner)
            .insert_resource(IterationCount(0))
            .insert_resource(PopulationStatistics::new())
            .add_systems(Startup, populate_person_world)
            .add_plugins(BreakupUpdateSchedulePlugin)
            // Update loop
            .add_systems(
                Update,
                plot_wealth_distribution_instantaneous.run_if(is_at_start),
            )
            .add_systems(
                RecordData,
                (
                    record_number_of_people,
                    record_age_distribution,
                    record_wealth_distribution,
                    record_max_wealth,
                    record_gini_coefficient,
                ),
            )
            .add_systems(
                UpdateData,
                (
                    //
                    age_people,
                    food_cost_system,
                    bank_investment_returns,
                    product_inflation_system,
                    subsidise_the_poor.after(food_cost_system),
                ),
            )
            .add_systems(
                DeleteEntities,
                (
                    //
                    people_die,
                ),
            )
            .add_systems(AddEntities, (people_born))
            //
            // End of iteration
            //
            .add_systems(Last, despawn_dead_people)
            .add_systems(Last, update_iteration_count)
            .add_systems(
                Update,
                (
                    plot_wealth_distribution_instantaneous.run_if(is_at_end),
                    plot_population_size.run_if(is_at_end),
                    plot_gini_coefficient.run_if(is_at_end),
                    plot_max_wealth.run_if(is_at_end),
                    // plot_age_distribution.run_if(is_at_end),
                    // plot_wealth_distribution.run_if(is_at_end),
                ),
            );
    }
}

// tests
#[cfg(test)]

// test to check that if people die the ancestors are born
mod tests {
    use super::*;

    fn one_old_person_world(mut commands: Commands) {
        let person = commands
            .spawn(PersonBundle {
                person: Person,
                bank_account: BankAccount { balance: 1000.0 },
                job: Job { salary: 1000.0 },
                age: Age(80),
            })
            .id();
    }

    fn check_death_event(mut death_event: EventReader<DeathEvent>) {
        let death_event = death_event.read().next();
        assert!(death_event.is_some());
    }

    #[test]
    fn test_people_born() {
        let mut app = App::new();
        app.add_event::<DeathEvent>()
            .insert_resource(IterationCount(0))
            .add_systems(Startup, one_old_person_world)
            // Update loop
            .add_systems(
                Update,
                (
                    age_people,
                    people_die.after(age_people),
                    people_born.after(people_die),
                    check_death_event.after(people_born),
                ),
            );

        app.update();
    }
}
