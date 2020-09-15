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
pub struct Airport{
    pub icao: String,
    pub lat: f32,
    pub lon: f32
}

impl Clone for Airport{
    fn clone(&self) -> Self { 
        Airport {icao: self.icao.clone(), lat: self.lat, lon: self.lon}
    }
}