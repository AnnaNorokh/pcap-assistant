use byteorder::{BigEndian, LittleEndian};


use crate::{
    Endianness,
    myerrors::*,
    pcap::myheader::PcapHeader,
    pcap::vpp_packet::*,
    peek_reader::PeekReader
};

use std::{io::Read, marker::PhantomData};


/// Wraps another reader and uses it to read a Pcap formated stream.
///
/// It implements the Iterator trait in order to read one packet at a time
///
/// # Examples
///
/// ```rust,no_run
/// use std::fs::File;
/// use pcap_file::pcap::PcapReader;
///
/// let file_in = File::open("test.pcap").expect("Error opening file");
/// let pcap_reader = PcapReader::new(file_in).unwrap();
///
/// // Read test.pcap
/// for pcap in pcap_reader {
///
///     //Check if there is no error
///     let pcap = pcap.unwrap();
///
///     //Do something
/// }
/// ```
#[derive(Debug)]
pub struct PcapReader<T:Read, P: SomePacket<'static>> {

    phantom_data: PhantomData<P>,
    pub header: PcapHeader,
    reader: PeekReader<T>
}

impl <T:Read, P: SomePacket<'static>> PcapReader<T, P>{

    /// Create a new PcapReader from an existing reader.
    /// This function read the global pcap header of the file to verify its integrity.
    ///
    /// The underlying reader must point to a valid pcap file/stream.
    ///
    /// # Errors
    /// Return an error if the data stream is not in a valid pcap file format.
    /// Or if the underlying data are not readable.
    ///
    /// # Examples
    /// ```rust,no_run
    /// use std::fs::File;
    /// use pcap_file::pcap::PcapReader;
    ///
    /// let file_in = File::open("test.pcap").expect("Error opening file");
    /// let pcap_reader = PcapReader::new(file_in).unwrap();
    /// ```
    pub fn new(mut reader:T) -> ResultParsing<Self> {

        Ok(
            Self {

                phantom_data: Default::default(),
                header : PcapHeader::from_reader(&mut reader)?,
                reader : PeekReader::new(reader)
            }
        )
    }

    /// Consumes the `PcapReader`, returning the wrapped reader.
    pub fn into_reader(self) -> T{
        self.reader.inner
    }

    /// Gets a reference to the underlying reader.
    ///
    /// It is not advised to directly read from the underlying reader.
    pub fn get_ref(&self) -> &T{
        &self.reader.inner
    }

    /// Gets a mutable reference to the underlying reader.
    ///
    /// It is not advised to directly read from the underlying reader.
    pub fn get_mut(&mut self) -> &mut T{
        &mut self.reader.inner
    }

}


impl <T: Read, P: SomePacket<'static>> Iterator for PcapReader<T, P> {

    type Item = ResultParsing<P::Item>;

    fn next(&mut self) -> Option<Self::Item> {

        match self.reader.is_empty() {
            Ok(is_empty) if is_empty => {
                return None;
            },
            Err(err) => return Some(Err(err.into())),
            _ => {}
        }

        let ts_resolution = self.header.ts_resolution();

        Some(
            match self.header.endianness() {
                Endianness::Big => P::from_reader::<_, BigEndian>(&mut self.reader, ts_resolution),
                Endianness::Little => P::from_reader::<_, LittleEndian>(&mut self.reader, ts_resolution)
            }
        )
    }

}
