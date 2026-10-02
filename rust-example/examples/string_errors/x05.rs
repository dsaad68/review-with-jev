use std::collections::HashMap;
use std::error::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Room {
    Atlas,
    Borealis,
    Cosmos,
}

struct Booking {
    room: Room,
    start_hour: u8,
    end_hour: u8,
    attendees: u32,
}

struct Calendar {
    capacity: HashMap<Room, u32>,
    bookings: Vec<Booking>,
}

impl Calendar {
    fn book(&mut self, booking: Booking) -> Result<usize, Box<dyn Error>> {
        if booking.start_hour >= booking.end_hour {
            return Err(format!("start {} is not before end {}", booking.start_hour, booking.end_hour).into());
        }
        if booking.start_hour < 8 || booking.end_hour > 20 {
            return Err(format!("{:?} is outside office hours", booking.room).into());
        }
        let capacity = self
            .capacity
            .get(&booking.room)
            .ok_or_else(|| format!("{:?} is not bookable", booking.room))?;
        if booking.attendees > *capacity {
            return Err(format!("{} attendees exceed capacity {capacity}", booking.attendees).into());
        }
        let clash = self.bookings.iter().any(|b| {
            b.room == booking.room && b.start_hour < booking.end_hour && booking.start_hour < b.end_hour
        });
        if clash {
            return Err(format!("{:?} is already booked then", booking.room).into());
        }
        self.bookings.push(booking);
        Ok(self.bookings.len())
    }
}

fn main() {
    let mut calendar = Calendar {
        capacity: HashMap::from([(Room::Atlas, 6), (Room::Borealis, 12)]),
        bookings: Vec::new(),
    };
    let requests = [
        Booking { room: Room::Atlas, start_hour: 9, end_hour: 10, attendees: 4 },
        Booking { room: Room::Atlas, start_hour: 9, end_hour: 11, attendees: 2 },
        Booking { room: Room::Borealis, start_hour: 14, end_hour: 13, attendees: 5 },
        Booking { room: Room::Borealis, start_hour: 15, end_hour: 16, attendees: 20 },
        Booking { room: Room::Cosmos, start_hour: 10, end_hour: 11, attendees: 3 },
        Booking { room: Room::Borealis, start_hour: 19, end_hour: 21, attendees: 8 },
    ];
    for request in requests {
        match calendar.book(request) {
            Ok(n) => println!("confirmed, {n} bookings today"),
            Err(e) => println!("rejected: {e}"),
        }
    }
}
