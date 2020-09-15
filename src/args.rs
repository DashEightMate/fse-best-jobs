extern crate structopt;

use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(name = "fse-best-jobs")]
pub enum Args{
    ///Query most profitable FSEconomy flights 
    Query{
        ///FSEconomy user data key, found in the server portal under Home - Data Feeds
        key: String,
        ///Name of airplane as it appears in FSEconomy
        ac_name: String,
        ///Fuel usage of plane, in gallons per hour
        gph: f64,
        ///Cruising speed of plane, in knots
        tas: i32,
        ///Fuel tank size of plane, in gallons
        max_fuel: i32,
        ///Maximum passengers of plane
        max_pax: i32,
        ///Maximum cargo amount of plane, in kg
        max_cargo: i32,
        ///System fuel price, found under "Local Market" in most airports in the server portal
        system_fuel: f64,
    }
}