/*
    "Best Job Finder" for FSEconomy.
    Takes the airplanes with the lowest rental costs, and compares their jobs to find the most profitable
    CLI arguments:
        - API key. needed to access fse api
        - Airplane name, as it appears in FSE
        - Airplane Gallons per Hour
        - Airplane True Air Speed
        - Airplane Fuel Tank size
        - Airplane Max Passengers
        - Airplane Max Cargo [kg]
        - System Fuel

    FSE Best Job Finder Copyright 2020 Logan Serino

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

extern crate quick_xml;
extern crate reqwest;
extern crate csv;
extern crate geoutils;
extern crate structopt;

mod plane;
mod airport;
mod job;
mod args;

use std::error::Error;
use quick_xml::Reader;
use quick_xml::events::Event;
use geoutils::Location;
use structopt::StructOpt;

use plane::Rentable;
use airport::Airport;
use job::Job;
use args::Args;

fn find_jobs(api_key: &String, airplane_name: &String, airplane_gph: f64, airplane_tas: i32,
    airplane_tank: i32, airplane_pax: i32, airplane_cargo: i32, system_fuel: f64) -> Result<(), Box<dyn Error>>{

    println!("Sending request for all planes of type {}", airplane_name);
    let airplane_resp = reqwest::blocking::get(
        &format!("https://server.fseconomy.net/data?userkey={}&format=xml&query=aircraft&search=makemodel&makemodel={}",
        api_key, airplane_name))?.text()?;
    //println!("{}", airplane_resp);
    println!("Received response. Parsing list...");
    let mut reader = Reader::from_str(&airplane_resp);
    reader.trim_text(true);
    
    let mut prev_tag = String::new();
    let mut planes = Vec::new();
    let mut buf = Vec::new();
    loop {
        match reader.read_event(&mut buf){
            Ok(Event::Start(ref e)) => {
                if e.name() == b"Aircraft" { //create a new plane if we reach a new plane tag
                    planes.push(plane::Plane {location: String::new(), dry_rental: None, wet_rental: None});
                }
                prev_tag = std::str::from_utf8(e.name()).unwrap().to_string(); //prev tag, for matching when we hit text
            },
            Ok(Event::Text(ref e)) => {
                let text = e.unescape_and_decode(&reader).unwrap();
                let mut last_plane = planes.last_mut().unwrap();
                match &prev_tag[..]{
                    "Location" => last_plane.location = text,
                    "RentalDry" => last_plane.dry_rental = parse_and_test_rental(text),
                    "RentalWet" => last_plane.wet_rental = parse_and_test_rental(text),
                    _ => ()
                }
            },
            Ok(Event::End(ref e)) => {
                if e.name() == b"Aircraft" {
                    let curr_plane = planes.pop().unwrap();
                    if (curr_plane.dry_rental.is_some() || curr_plane.wet_rental.is_some())
                        && curr_plane.location != "In Flight"{
                        if planes.len() < 3 {planes.push(curr_plane)}
                        else {
                            let curr_avg = &curr_plane.get_rental_avg();
                            for plane in &planes{
                                //println!("Comparing rental {} to {}", curr_avg, &plane.get_rental_avg());
                                if curr_avg < &plane.get_rental_avg(){
                                    let old_iter = planes.iter().position(|r| r == &plane).unwrap();
                                    //println!("Plane rental average {} less than compare. Replacing at {}", curr_avg, old_iter);
                                    planes.remove(old_iter);
                                    //println!("Cloning dry {:?}. wet {:?}", curr_plane.dry_rental, curr_plane.wet_rental);
                                    planes.push(curr_plane.clone());
                                    break;
                                }
                            }
                        }
                    }
                }
            }
            Ok(Event::Eof) => break,
            _ => ()
        }
    }

    let mut locations = Vec::new();
    for plane in &planes {
        locations.push(plane.location.clone());
    }
    
    println!("Cheapest planes found. Requesting job info for airports of planes...");
    let airports_resp = reqwest::blocking::get(
        &format!("https://server.fseconomy.net/data?userkey={}&format=xml&query=icao&search=jobsfrom&icaos={}",
        api_key, locations.join("-")))?.text()?;
    println!("Received job info response. Parsing...");
    
    let mut reader = Reader::from_str(&airports_resp);
    reader.trim_text(true);

    let mut prev_tag = String::new();
    let mut remove = false;
    let mut jobs = Vec::new();
    let mut buf = Vec::new();

    loop {
        match reader.read_event(&mut buf){
            Ok(Event::Start(ref e)) => {
                if e.name() == b"Assignment"{
                    jobs.push(Job::new());
                }
                prev_tag = std::str::from_utf8(e.name()).unwrap().to_string();
            },
            Ok(Event::Text(ref e)) => {
                let text = e.unescape_and_decode(&reader).unwrap();
                let mut last_job = jobs.last_mut().unwrap();
                match &prev_tag[..]{
                    "ToIcao" => {
                        last_job.to_icao = text.clone();
                        locations.push(text);
                    }
                    "FromIcao" => last_job.from_icao = text,
                    "Amount" => last_job.amount = text.parse().unwrap(),
                    "Commodity" => if text == "Cargo" {last_job.cargo = true;},
                    "Pay" => last_job.price = text.parse().unwrap(),
                    "Type" => if text == "VIP" {last_job.vip = true;},
                    "AircraftId" => if text != "0" {remove = true;},
                    _ => ()
                }
            },
            Ok(Event::End(ref e)) => {
                if e.name() == b"Assignment"{
                    if remove {jobs.pop();}
                }
            },
            Ok(Event::Eof) => break,
            _ => ()
        }
    }
    
    println!("Found airports of planes, reading FSE airport database...");
    //println!("{:?}", &locations);
    let airports_opt = find_airport_info("icaodata.csv", &locations);
    let mut airports_unwrapped = Vec::new();
    match airports_opt{
        None => eprintln!("Csv database not found!"),
        Some(val) => {
            for airport_opt in val{
                match airport_opt{
                    //I would put a message here, but there are some weird mystery None's that show up in the list,
                    //even though all jobs can find distances.
                    None => (),
                    Some(airport) => airports_unwrapped.push(airport),
                }
            }
        }
    }
    //println!("{:?}", airports_unwrapped);
    println!("Airports collected from db. Calculating job distances...");
    for job in &mut jobs{
        let from_icao = airports_unwrapped.iter().find(|r| r.icao == job.from_icao);
        let to_icao = airports_unwrapped.iter().find(|r| r.icao == job.to_icao);
        if from_icao.is_some() && to_icao.is_some(){
            job.dist = Some(Location::new(from_icao.unwrap().lat, from_icao.unwrap().lon)
                .distance_to(&Location::new(to_icao.unwrap().lat, to_icao.unwrap().lon))
                .unwrap().meters() / 1852.0); //meters to nm - divide by 1852
        }
        else {println!("Airport for job (from: {}, to: {}) not found, skipping dist check",
             job.from_icao, job.to_icao);}
    }

    println!("Finding combinational jobs...");
    let blank_job = Job::new(); //needed for the while loop below
    let mut job_iter = jobs.iter();
    let mut combination_jobs = Vec::new();
    let mut prev_comb = job_iter.next().unwrap();
    while prev_comb.vip {prev_comb = job_iter.next().unwrap_or(&blank_job)}
    loop {
        match job_iter.next(){
            Some(val) => {
                if val.from_icao == prev_comb.from_icao && val.to_icao == prev_comb.to_icao && val.cargo == prev_comb.cargo && !val.vip{
                    let new_comb = Job {from_icao: val.from_icao.clone(), to_icao: val.to_icao.clone(), commodity: val.commodity.clone(),
                        amount: val.amount+prev_comb.amount, cargo: val.cargo, dist: val.dist, price: val.price+prev_comb.price, vip: false, profit: None};
                    combination_jobs.push(new_comb);
                    prev_comb = combination_jobs.last().unwrap();
                } else {
                    prev_comb = val;
                }
            },
            None => break,
        }
    }
    jobs.append(&mut combination_jobs);

    println!("Finding best profits...");
    for job in &mut jobs{
        let plane_opt = planes.iter().find(|r| r.location == job.from_icao);
        if job.dist.is_some() && plane_opt.is_some(){
            let plane = plane_opt.unwrap();
            if (job.cargo && job.amount <= airplane_cargo) || (!job.cargo && job.amount <= airplane_pax){
                //println!("Job amount less than plane amount");
                let time = job.dist.unwrap() / airplane_tas as f64;
                let fuel_needed = time*airplane_gph;
                if (airplane_tank as f64) > fuel_needed{
                    //println!("Airplane tank large enough");
                    let fuel_price = fuel_needed * system_fuel;
                    let mut profits: (f64, f64) = (0.0, 0.0);
                    if plane.dry_rental.is_some(){
                        profits.0 = job.price as f64 - (plane.dry_rental.unwrap() as f64) * time + fuel_price;
                    } if plane.wet_rental.is_some(){
                        profits.0 = job.price as f64 - (plane.wet_rental.unwrap() as f64) * time;
                    }
                    if profits.0 != 0.0 && profits.1 != 0.0{
                        match profits.0 > profits.1{
                            true => job.profit = Some(job::JobResult {profit: profits.0, dry: true}),
                            false => job.profit = Some(job::JobResult {profit: profits.1, dry: false})
                        }
                    } else {
                        if profits.0 != 0.0 {job.profit = Some(job::JobResult {profit: profits.0, dry: true})}
                        else if profits.1 != 0.0 {job.profit = Some(job::JobResult {profit: profits.1, dry: false})}
                    }
                }
            }
        }
    }
    let mut filter_jobs: Vec<Job> = jobs.into_iter().filter(|f| f.profit.is_some()).collect();
    filter_jobs.sort_by(|a, b| b.profit.as_ref().unwrap().profit.partial_cmp(&a.profit.as_ref().unwrap().profit).unwrap());

    println!("{:?}", filter_jobs.drain(0..3));

    Ok(())
}

fn parse_and_test_rental(to_parse: String) -> Option<f32>{
    let rental: Option<f32>;
    match &to_parse.parse::<f32>(){
        Ok(float) => {
            //println!("Parsed rental price {}", float);
            if float < &0.01{
                rental = None;
            } else {
                rental = Some(*float);
            }
        }
        Err(_e) => rental = None
    }
    return rental;
}

fn find_airport_info(path: &str, icao: &Vec<String>) -> Option<Vec<Option<Airport>>>{
    let mut output: Vec<Option<Airport>> = vec![None; icao.len()]; //create vec with size of icao with all None
    let mut reader = csv::Reader::from_path(path).ok()?;
    for line in reader.records(){
        match line{
            Ok(val) => {
                //get position in icao list of csv icao (if it's present in icao list)
                let pos = icao.iter().position(|r| r == val.get(0).unwrap());
                if pos.is_some() {
                    output[pos.unwrap()] = Some(Airport {icao: val.get(0).unwrap().to_string(),
                        lat: val.get(1).unwrap().parse().unwrap(), lon: val.get(2).unwrap().parse().unwrap()});
                    
                    //println!("Found airport {} in db, placing in pos {}. {:?}", icao[pos.unwrap()], pos.unwrap(), output[pos.unwrap()]);
                }
            },
            Err(_e) => break
        }
    }
    return Some(output);
}

fn main() {
    println!("FSE Best Job Finder  Copyright (C) 2020  Logan Serino
    This program comes with ABSOLUTELY NO WARRANTY.
    This is free software, and you are welcome to redistribute it
    under certain conditions. Reference the COPYING file
    inside the program source for details.");
    match Args::from_args(){
        Args::Query{key, ac_name, gph, tas,
             max_fuel, max_pax, max_cargo, system_fuel} => {
                match find_jobs(&key, &ac_name, gph, tas,
                    max_fuel, max_pax, max_cargo, system_fuel){
                    Err(e) => println!("Error while finding jobs: {}", e),
                    Ok(_o) => ()
                }
        }
    }
}