/*
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

#[derive(Debug)]
pub struct Job{
    pub to_icao: String,
    pub from_icao: String,
    pub commodity: String,
    pub amount: i32,
    pub cargo: bool,
    pub dist: Option<f64>,
    pub price: f32,
    pub vip: bool,
    pub profit: Option<JobResult>,
}

#[derive(Debug)]
pub struct JobResult{
    pub profit: f64,
    pub dry: bool
}

impl Job{
    pub const fn new() -> Job{
        Job {to_icao: String::new(), from_icao: String::new(), commodity: String::new(),
            amount: 0, cargo: false, dist: None, price: 0.0, vip: false, profit: None}
    }
}