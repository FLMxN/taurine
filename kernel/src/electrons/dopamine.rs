use core3::io::{Cursor, Read as CoreRead, Seek as CoreSeek, SeekFrom as CoreSeekFrom, Write as CoreWrite};
use fatfs::{FileSystem, FormatVolumeOptions, FsOptions, IoBase, Read, ReadWriteSeek, Seek, SeekFrom, Write};
use crate::{read_tsc, u64_to_bytes};
// use crate::Mutex;

pub struct FATBuffer<'a>(pub Cursor<&'a mut [u8]>);

impl<'a> IoBase for FATBuffer<'a> {
    type Error = ();
}

impl<'a> Read for FATBuffer<'a> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        self.0.read(buf).map_err(|_| ())
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), Self::Error> {
        self.0.read_exact(buf).map_err(|_| ())
    }
}

impl<'a> Write for FATBuffer<'a> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        self.0.write(buf).map_err(|_| ())
    }

    fn write_all(&mut self, buf: &[u8]) -> Result<(), Self::Error> {
        self.0.write_all(buf).map_err(|_| ())
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        self.0.flush().map_err(|_| ())
    }
}

impl<'a> Seek for FATBuffer<'a> {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, Self::Error> {
        let core_pos = match pos {
            SeekFrom::Start(n) => CoreSeekFrom::Start(n),
            SeekFrom::End(n) => CoreSeekFrom::End(n),
            SeekFrom::Current(n) => CoreSeekFrom::Current(n),
        };
        self.0.seek(core_pos).map_err(|_| ())
    }
}


pub fn recall<IO>(
    fs: &FileSystem<IO>,
    path: &str,
    output: &mut [u8],
) -> Result<usize, fatfs::Error<IO::Error>>
where
    IO: ReadWriteSeek,
{
    let root = fs.root_dir();
    let mut file = root.open_file(path)?;
    let mut read = 0;

    while read < output.len() {
        let count = file.read(&mut output[read..])?;
        if count == 0 {
            break;
        }
        read += count;
    }

    Ok(read)
}


pub fn internalize<IO>(
    fs: &FileSystem<IO>,
    path: &str,
    contents: &[u8],
) -> Result<(), fatfs::Error<IO::Error>>
where
    IO: ReadWriteSeek,
{
    let root = fs.root_dir();
    let mut file = root.create_file(path)?;

    file.truncate()?;
    file.write_all(contents)?;
    file.flush()?;
    Ok(())
}