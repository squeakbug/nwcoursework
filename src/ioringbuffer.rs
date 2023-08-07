use std::io;
use std::io::{Read, Write};

/// Собственная "dummy" реализация циклического буфера
/// для записи из потока чтения и чтения из потока записи
/// в буфер без дополнительного копирования из промежуточного буфера
/// (Данная реализация контролирует границы записи)
/// writei и readi указывают на одну ячейку (filled = writei - readi)
/// writei - индекс, по которому записываются данные
/// readi - индекс, по которму считываются данные
pub struct Ioribnbuffer<const SIZE: usize> {
    writei: usize,
    readi: usize,
    buff: [u8; SIZE],
}

impl<const SIZE: usize> Ioribnbuffer<SIZE> {
    pub fn new() -> Self {
        Self {
            writei: 0,
            readi: 0,
            buff: [0u8; SIZE],
        }
    }

    pub fn write(&mut self, stream: &mut dyn Read) -> io::Result<usize> {
        if self.writei >= self.readi {
            let rc = stream.read(&mut self.buff[self.writei..SIZE])?;
            self.writei = (self.writei + rc) % SIZE;
            return Ok(rc);
        } else {
            let rc = stream.read(&mut self.buff[self.writei..self.readi])?;
            self.writei = (self.writei + rc) % SIZE;
            return Ok(rc);
        }
    }

    pub fn read(&mut self, stream: &mut dyn Write) -> io::Result<usize> {
        if self.readi > self.writei {
            let rc = stream.write(&self.buff[self.readi..SIZE])?;
            self.readi = (self.readi + rc) % SIZE;
            return Ok(rc);
        } else {
            let rc = stream.write(&self.buff[self.readi..self.writei])?;
            self.readi = (self.readi + rc) % SIZE;
            return Ok(rc);
        }
    }

    pub fn len(&self) -> usize {
        if self.writei >= self.readi {
            self.writei - self.readi
        } else {
            SIZE - self.readi + self.writei
        }
    }
}
