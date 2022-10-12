use crate::RandomReader;

use std::any::Any;
use std::io::{IoSlice, IoSliceMut, Read, SeekFrom};
use std::time::SystemTime;

use anyhow::{anyhow, Context};
use wasi_common::file::*;
use wasi_common::{Error, SystemTimeSpec, WasiFile};

pub struct RandomDevice {
    flags: FdFlags,
    access_time: SystemTime,
    modify_time: SystemTime,
    create_time: SystemTime,
}

impl Default for RandomDevice {
    fn default() -> Self {
        let now = SystemTime::now();

        Self {
            flags: FdFlags::empty(),
            access_time: now,
            modify_time: now,
            create_time: now,
        }
    }
}

#[async_trait::async_trait]
impl WasiFile for RandomDevice {
    fn as_any(&self) -> &dyn Any {
        self
    }

    async fn get_filetype(&mut self) -> Result<FileType, Error> {
        Ok(FileType::RegularFile)
    }

    async fn datasync(&mut self) -> Result<(), Error> {
        Ok(())
    }

    async fn sync(&mut self) -> Result<(), Error> {
        Ok(())
    }

    async fn get_fdflags(&mut self) -> Result<FdFlags, Error> {
        Ok(self.flags)
    }

    async fn set_fdflags(&mut self, _flags: FdFlags) -> Result<(), Error> {
        Err(anyhow!("Read only device"))
    }

    async fn get_filestat(&mut self) -> Result<Filestat, Error> {
        Ok(Filestat {
            filetype: FileType::SocketDgram,
            device_id: 0,
            inode: 0,
            nlink: 0,
            size: 0,
            atim: Some(self.access_time),
            mtim: Some(self.modify_time),
            ctim: Some(self.create_time),
        })
    }

    async fn set_filestat_size(&mut self, _size: u64) -> Result<(), Error> {
        Err(anyhow!("read only device"))
    }

    async fn advise(&mut self, _offset: u64, _len: u64, _advice: Advice) -> Result<(), Error> {
        Ok(())
    }

    async fn allocate(&mut self, _offset: u64, _size: u64) -> Result<(), Error> {
        Err(anyhow!("read only device"))
    }

    async fn set_times(
        &mut self,
        _atime: Option<SystemTimeSpec>,
        _mtime: Option<SystemTimeSpec>,
    ) -> Result<(), Error> {
        Err(anyhow!("read only device"))
    }

    async fn read_vectored<'a>(
        &mut self,
        bufs: &mut [std::io::IoSliceMut<'a>],
    ) -> Result<u64, Error> {
        let total_read: usize = bufs.iter().map(|b| b.len()).sum();

        for buf in bufs {
            let mut read_size = 0;

            while read_size != buf.len() {
                read_size = RandomReader
                    .read(&mut buf[read_size..])
                    .with_context(|| "unexpected error getting random bytes")?;
            }
        }

        Ok(total_read as u64)
    }

    async fn read_vectored_at<'a>(
        &mut self,
        bufs: &mut [std::io::IoSliceMut<'a>],
        _offset: u64,
    ) -> Result<u64, Error> {
        self.read_vectored(bufs).await
    }

    async fn write_vectored<'a>(&mut self, bufs: &[IoSlice<'a>]) -> Result<u64, Error> {
        self.write_vectored_at(bufs, 0).await
    }

    async fn write_vectored_at<'a>(
        &mut self,
        bufs: &[std::io::IoSlice<'a>],
        _offset: u64,
    ) -> Result<u64, Error> {
        let n: usize = bufs.iter().map(|b| b.len()).sum();
        Ok(n as u64)
    }

    async fn seek(&mut self, _pos: SeekFrom) -> Result<u64, Error> {
        Ok(0)
    }

    async fn peek(&mut self, buf: &mut [u8]) -> Result<u64, Error> {
        let mut slices = [IoSliceMut::new(buf)];
        Ok(self.read_vectored(&mut slices).await?)
    }

    async fn num_ready_bytes(&self) -> Result<u64, Error> {
        Ok(0)
    }

    async fn readable(&self) -> Result<(), Error> {
        Ok(())
    }

    async fn writable(&self) -> Result<(), Error> {
        Ok(())
    }
}

#[tokio::test]
async fn read_vectored_test() {
    const LENGTH: usize = 10_000;
    let mut device = RandomDevice::default();
    let mut buffer = [0; LENGTH];
    let mut slices = [IoSliceMut::new(&mut buffer)];
    assert_eq!(device.read_vectored(&mut slices).await.unwrap() as usize, LENGTH);
    assert_ne!(buffer, [0; LENGTH]);
}
