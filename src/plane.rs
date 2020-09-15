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

use std::cmp::PartialEq;

#[derive(Debug)]
pub struct Plane {
    pub location: String,
    pub dry_rental: Option<f32>,
    pub wet_rental: Option<f32>,
}

pub trait Rentable{
    fn get_rental_avg(&self) -> f32;
}

impl Rentable for Plane{
    fn get_rental_avg(&self) -> f32{
        if self.dry_rental.is_none() && self.wet_rental.is_none() {
            return 0.0
        }
        return (self.dry_rental.unwrap_or(0.0) + self.wet_rental.unwrap_or(0.0)) / 
            (self.dry_rental.is_some() as i8 as f32 + self.wet_rental.is_some() as i8 as f32)
    }
}

impl PartialEq for &Plane{
    fn eq(&self, other: &Self) -> bool { 
        self.location == other.location &&
         approx_float_eq(&self.dry_rental.unwrap_or_default(), &other.dry_rental.unwrap_or_default()) &&
         approx_float_eq(&self.wet_rental.unwrap_or_default(), &other.wet_rental.unwrap_or_default())
    }
}

impl Clone for Plane{
    fn clone(&self) -> Self { 
        //println!("Copying dry {:?}, wet {:?}", self.dry_rental, self.wet_rental);
        return Plane {location: self.location.clone(), dry_rental: self.dry_rental, wet_rental:self.wet_rental}
    }
}

fn approx_float_eq(val1: &f32, val2: &f32) -> bool{
    if (val1 - val2).abs() < 0.1 {
        return true
    }
    return false
}