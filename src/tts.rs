use winapi;
use ole32;
use std::iter::Extend;
use std::mem;
use std::thread;
use std::sync::mpsc::{sync_channel, SyncSender, Receiver};
use self::SpeechMessage::*;

pub struct Speechifier {
    mailbox: Option<SyncSender<SpeechMessage>>,
}

enum SpeechMessage {
    Word(String),
    Stop,
}

impl Speechifier {
    pub fn new() -> Speechifier {
        Speechifier {
            mailbox: None,
        }
    }

    pub fn start(&mut self) {
        let (tx, rx) = sync_channel(0);
        self.mailbox = Some(tx);

        thread::spawn(move || {
            unsafe { speechify(rx); }
        });
    }

    pub fn stop(&mut self) {
        if let Some(ref mailbox) = self.mailbox {
            mailbox.send(Stop).unwrap();
        }

        self.mailbox = None;
    }

    pub fn queue(&self, word: String) {
        if let Some(ref mailbox) = self.mailbox {
            mailbox.send(Word(word)).unwrap();
        }
    }
}

#[inline]
fn failed(hr: winapi::HRESULT) -> bool {
    hr < 0
}

#[inline]
fn succeeded(hr: winapi::HRESULT) -> bool {
    !failed(hr)
}

unsafe fn speechify(rx: Receiver<SpeechMessage>) {
    let mut hr;
    let mut voice: *mut winapi::ISpVoice = mem::zeroed();

    hr = ole32::CoInitialize(mem::zeroed());
    if failed(hr) {
        return;
    }

    let sp_voice: Vec<_> = "SAPI.SpVoice\0".utf16_units().collect();
    let mut clsid_spvoice: winapi::CLSID = mem::zeroed();
    hr = ole32::CLSIDFromProgID(&sp_voice[0], &mut clsid_spvoice);
    if failed(hr) {
        return;
    }

    hr = ole32::CoCreateInstance(
        &clsid_spvoice,
        mem::zeroed(),
        winapi::CLSCTX_ALL,
        &winapi::UuidOfISpVoice,
        &mut voice as *mut *mut winapi::ISpVoice as *mut *mut winapi::c_void
    );

    if succeeded(hr) {
        (*voice).SetRate(2);
        speech_loop(rx, &mut *voice);
        (*voice).Release();
    }

    ole32::CoUninitialize();
}

unsafe fn speech_loop(rx: Receiver<SpeechMessage>, voice: &mut winapi::ISpVoice) {
    let mut buffer: Vec<u16> = Vec::new();
    loop {
        if let Ok(Word(word)) = rx.recv() {
            buffer.extend(word.utf16_units());
            buffer.push(0);
        } else {
            return;
        }

        voice.Speak(&buffer[0], 0, mem::zeroed());
        voice.WaitUntilDone(winapi::INFINITE);

        buffer.clear();
    }
}
