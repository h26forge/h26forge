use crate::common::data_structures::NALUheader;
use crate::common::data_structures::StapA;
use crate::common::data_structures::StapB;
use crate::common::data_structures::Mtap16;
use crate::common::data_structures::Mtap24;
use crate::encoder::binarization_functions::generate_fixed_length_value;
use std::fs::File;
use std::io::prelude::*;


pub const SAFESTART_RTP_0: [u8; 15] = [
    0x67, 0x42, 0xC0, 0x15, 0x8C, 0x8D, 0x40, 0xA0, 0xCB, 0xCF, 0x00, 0xF0, 0x88, 0x46, 0xA0,
];
pub const SAFESTART_RTP_1: [u8; 4] = [0x68, 0xCE, 0x3C, 0x80];
pub const SAFESTART_RTP_2: [u8; 908] = [
    0x65, 0xB8, 0x00, 0x04, 0x00, 0x00, 0x05, 0x39, 0x31, 0x40, 0x00, 0x40, 0xD2, 0x4E, 0x4E, 0x4E,
    0x4E, 0x4E, 0x4E, 0x4E, 0x4E, 0x4E, 0x4E, 0x4E, 0x4E, 0x4E, 0x4E, 0x4E, 0x4E, 0x4E, 0x4E, 0x4E,
    0xBA, 0xEB, 0xAE, 0xBA, 0xEB, 0xAE, 0xBA, 0xEB, 0xAE, 0xBA, 0xEB, 0xAE, 0xBA, 0xEB, 0xAE, 0xBA,
    0xFF, 0xFF, 0xE1, 0x05, 0xD8, 0xA0, 0x00, 0x20, 0x0B, 0xC2, 0x26, 0x64, 0xF0, 0x78, 0x1B, 0x1D,
    0x4E, 0xA5, 0xC4, 0xA9, 0xAD, 0xAD, 0xAD, 0xAD, 0xAD, 0xAD, 0xAD, 0xAF, 0xFE, 0x3E, 0x70, 0x40,
    0xBB, 0x01, 0x30, 0x70, 0xDF, 0xEE, 0xBE, 0x13, 0x2D, 0xE0, 0x79, 0x8A, 0xE7, 0xA7, 0xAE, 0xBA,
    0xEB, 0xAE, 0xBA, 0xEB, 0xFF, 0x87, 0xFA, 0x05, 0x98, 0x1B, 0x35, 0x2D, 0xC0, 0x5B, 0x02, 0x82,
    0x46, 0xD2, 0xFE, 0x2D, 0x72, 0x23, 0xA7, 0x1A, 0x31, 0xAD, 0xB6, 0xF6, 0xB6, 0xB6, 0xB6, 0xB6,
    0xB6, 0xB6, 0xB6, 0xBF, 0xF0, 0xFF, 0x40, 0xB3, 0x01, 0x33, 0x74, 0xFC, 0xF8, 0xA9, 0xB0, 0xBD,
    0xA3, 0x6F, 0x4F, 0xFF, 0xF8, 0x06, 0xC3, 0xB8, 0x04, 0xD6, 0xE2, 0x71, 0xF7, 0xBC, 0x3E, 0xF8,
    0x1F, 0x30, 0x80, 0x9C, 0xA6, 0xF2, 0x6E, 0x16, 0x6E, 0x07, 0x4C, 0xE5, 0x9D, 0x5C, 0xA9, 0x6B,
    0x6B, 0x6B, 0xFF, 0x8F, 0x58, 0x60, 0xBA, 0x09, 0x0E, 0xD2, 0x7E, 0xA9, 0x06, 0x80, 0x8E, 0x4D,
    0x3C, 0x30, 0x3A, 0x67, 0x2C, 0x0B, 0xCB, 0xC8, 0x3E, 0x2B, 0xD7, 0x5D, 0x75, 0xD7, 0x5D, 0x75,
    0xD7, 0x5D, 0x75, 0xD2, 0xC5, 0x3A, 0x62, 0x78, 0xFA, 0xEB, 0xFD, 0x2B, 0xB5, 0xB5, 0xB5, 0x8A,
    0x7E, 0x8F, 0x7F, 0xE6, 0xA9, 0xEB, 0xAE, 0xBA, 0xEB, 0xAE, 0xBA, 0xEB, 0xAE, 0xBA, 0xEB, 0xA7,
    0x75, 0xD7, 0x5D, 0x74, 0xF4, 0xF5, 0xD7, 0x5D, 0x75, 0xD7, 0x5D, 0x75, 0xD7, 0x5D, 0x75, 0xD3,
    0xD7, 0x5D, 0x75, 0xD3, 0xD3, 0xD7, 0x5D, 0x75, 0xD7, 0x5D, 0x75, 0xD7, 0x5D, 0x75, 0xD7, 0x4F,
    0x5D, 0x75, 0xD7, 0x4F, 0x4F, 0x5D, 0x75, 0xD7, 0x5D, 0x75, 0xD7, 0x5D, 0x75, 0xD7, 0x5D, 0x3D,
    0x75, 0xD7, 0x5D, 0x3D, 0x3D, 0x75, 0xD7, 0x5D, 0x75, 0xD7, 0x5D, 0x75, 0xD7, 0x5D, 0x74, 0xF5,
    0xD7, 0x5D, 0x74, 0xF4, 0xFF, 0xE3, 0x8E, 0xAF, 0xDF, 0x80, 0x0B, 0xB7, 0x71, 0xA2, 0xC4, 0xE6,
    0xFE, 0xED, 0x9D, 0xC6, 0x9A, 0xAD, 0x97, 0x65, 0x0A, 0x7B, 0x28, 0x86, 0xEB, 0xFF, 0xF5, 0xFB,
    0xEE, 0xBF, 0x1F, 0xBB, 0x90, 0xB5, 0x47, 0x1C, 0xD7, 0xF8, 0xFC, 0x02, 0xFA, 0x78, 0xE0, 0xF5,
    0x86, 0x2D, 0x20, 0xBA, 0x80, 0x3F, 0xB4, 0xFA, 0x40, 0x76, 0x15, 0x93, 0x6E, 0xD4, 0xF7, 0x3C,
    0x10, 0xD7, 0x1A, 0x29, 0x7A, 0xFF, 0x18, 0xC7, 0xFF, 0x4D, 0xF3, 0x33, 0x7F, 0xF1, 0xB5, 0x3F,
    0xF4, 0xD8, 0xE1, 0xD4, 0xB1, 0x86, 0xBF, 0x46, 0xE7, 0x7F, 0xE3, 0xF8, 0xFE, 0xF8, 0x31, 0xEF,
    0xBE, 0xA7, 0xEE, 0xFC, 0x77, 0x26, 0x36, 0x53, 0xB0, 0xDB, 0x0A, 0x75, 0xDD, 0xC3, 0xFE, 0x1E,
    0x82, 0x8E, 0xD1, 0x6A, 0x3B, 0xBE, 0x53, 0x86, 0xE7, 0x7B, 0xF2, 0xCB, 0x71, 0xFF, 0xBB, 0xF5,
    0xFE, 0x73, 0xFA, 0xBE, 0xDC, 0xEF, 0xFD, 0xFD, 0xBF, 0xFF, 0xBF, 0xFB, 0xE2, 0xAE, 0xF5, 0x20,
    0xEC, 0xAC, 0x32, 0x2E, 0x53, 0x87, 0xF5, 0xB6, 0x1D, 0x75, 0x54, 0xEB, 0xEE, 0x82, 0x79, 0xEA,
    0x6D, 0x49, 0x87, 0x8A, 0x9C, 0x60, 0x68, 0x1F, 0x57, 0x26, 0x07, 0xFE, 0x8A, 0xFF, 0x9A, 0xD6,
    0xAF, 0x15, 0x55, 0x93, 0xE5, 0x6B, 0x75, 0x2F, 0xF1, 0xF8, 0x01, 0xAB, 0xF0, 0xF4, 0x69, 0xD7,
    0x14, 0x06, 0x21, 0x77, 0xFE, 0x85, 0x01, 0xBF, 0x64, 0x85, 0xBF, 0xFF, 0x4E, 0xA2, 0xC6, 0xC8,
    0x82, 0x8A, 0xC6, 0x2E, 0xAC, 0xAD, 0x52, 0xA2, 0x6F, 0xEF, 0xDB, 0x22, 0x55, 0x4A, 0x45, 0xEB,
    0xF7, 0xFB, 0x65, 0xE4, 0xB1, 0x1E, 0xBC, 0x83, 0x73, 0x7F, 0x1F, 0x80, 0x37, 0x6A, 0x5F, 0x6B,
    0x01, 0xB6, 0x6D, 0xAD, 0x78, 0x14, 0xDD, 0x75, 0xBE, 0xFF, 0xF7, 0xF8, 0xAD, 0x67, 0x77, 0x0D,
    0x75, 0x36, 0x7F, 0xDF, 0xA0, 0x84, 0x67, 0x69, 0x50, 0x2E, 0x3E, 0x33, 0xFB, 0xF3, 0x03, 0x5F,
    0xA1, 0x6C, 0xE5, 0x45, 0x52, 0x3B, 0x37, 0xDF, 0xE3, 0xF0, 0xD2, 0xD4, 0x63, 0xB9, 0x3D, 0xC8,
    0x73, 0x66, 0x52, 0xE2, 0xF4, 0x6E, 0xFC, 0x31, 0xE4, 0x17, 0xB1, 0xFF, 0xFF, 0xEC, 0x3A, 0x65,
    0x1B, 0x71, 0xBA, 0xDA, 0xE9, 0x98, 0xEE, 0xE6, 0x35, 0xE5, 0xEF, 0xE6, 0x9F, 0xFB, 0x8D, 0xDD,
    0x5A, 0xDF, 0xFF, 0x40, 0x9B, 0xBF, 0x9A, 0x37, 0x6F, 0xFF, 0x98, 0xE0, 0xAF, 0xDB, 0xDF, 0x15,
    0x53, 0x8C, 0x4A, 0x8E, 0xA6, 0x28, 0xAB, 0x26, 0x3D, 0x5B, 0x31, 0x2B, 0xB6, 0x2C, 0x30, 0xAE,
    0x21, 0x47, 0x7D, 0xD8, 0x8B, 0xA6, 0x68, 0x2E, 0x52, 0x8A, 0xA3, 0xDF, 0x57, 0xF6, 0x8C, 0x6E,
    0x1E, 0x6A, 0xC3, 0x94, 0xB3, 0xDD, 0xD7, 0x6F, 0xF1, 0xF8, 0x01, 0xF7, 0x0B, 0xEB, 0x5C, 0xDD,
    0x6F, 0x15, 0xA5, 0x38, 0xBF, 0x9C, 0x66, 0xB7, 0xA9, 0xC7, 0x2B, 0x98, 0xBC, 0x68, 0x88, 0x24,
    0xFB, 0x9D, 0xB3, 0xFD, 0xB9, 0x9C, 0x16, 0xDF, 0x5E, 0x5A, 0x1C, 0xBF, 0xF9, 0x9F, 0x4B, 0x8D,
    0xC1, 0x22, 0x29, 0x7F, 0x8E, 0xC0, 0x10, 0xE6, 0xA0, 0x24, 0x15, 0x07, 0x57, 0x29, 0xBB, 0x00,
    0x31, 0x5C, 0xB2, 0xBF, 0xDD, 0x85, 0x8B, 0xEC, 0xA1, 0x7E, 0xF8, 0x10, 0xF1, 0x67, 0xE1, 0x4F,
    0xF1, 0xE6, 0x6C, 0xA3, 0x8D, 0x56, 0xFE, 0x29, 0x4B, 0x55, 0xC7, 0xDD, 0x02, 0x6E, 0xFF, 0x0D,
    0x1B, 0xB7, 0xFF, 0xCF, 0x8C, 0x5F, 0xFB, 0xE1, 0x5F, 0x77, 0xA6, 0xBE, 0xBC, 0x3B, 0x9B, 0x49,
    0xC7, 0xB1, 0x9A, 0xDF, 0xD0, 0xE3, 0x94, 0x65, 0x15, 0x7D, 0x3B, 0x3E, 0x76, 0xF4, 0x2A, 0xD8,
    0xD0, 0xD9, 0x6A, 0xDF, 0x7F, 0xED, 0xA6, 0x91, 0xDD, 0xDC, 0x6B, 0xC9, 0xC1, 0xFF, 0xDA, 0xBF,
    0xE3, 0xB0, 0x04, 0x52, 0x6D, 0x36, 0x83, 0xAB, 0x9E, 0x10, 0x9C, 0x66, 0x5F, 0xBE, 0x36, 0x7B,
    0xF9, 0xBF, 0xFE, 0xBE, 0x1F, 0xF3, 0xC0, 0x18, 0x77, 0xD0, 0x5B, 0x4E, 0x01, 0xC2, 0x12, 0x47,
    0xB8, 0xAB, 0xC3, 0xB6, 0xD0, 0x9C, 0x03, 0x2C, 0xEB, 0x7B, 0x5B, 0x5B, 0x5F, 0xC3, 0xE3, 0xCA,
    0xC1, 0x77, 0x22, 0x08, 0x2E, 0x82, 0x8B, 0xBE, 0x2A, 0xD1, 0x61, 0x70, 0x0C, 0xB0, 0x55, 0x53,
    0x0F, 0x27, 0xFF, 0xFF, 0xE1, 0xEE, 0x28, 0x00, 0x08, 0x06, 0xEF, 0x15, 0xBE, 0xFB, 0xEF, 0xBE,
    0xFB, 0xEF, 0xBE, 0xFB, 0xED, 0x6D, 0x6D, 0x6D, 0x6D, 0x6D, 0x6F, 0xC0,
];
pub const SAFESTART_RTP_3: [u8; 16] = [
    0x61, 0xE0, 0x0, 0x40, 0x0, 0xBE, 0x40, 0x53, 0x80, 0xCF, 0xE4, 0xEA, 0xE3, 0x4E, 0xF0, 0xAC,
];
pub const SAFESTART_RTP_4: [u8; 13] = [
    0x61, 0xE0, 0x0, 0x80, 0x01, 0x3E, 0x40, 0xEE, 0x03, 0x70, 0xEF, 0x0A, 0xC0,
];
pub const SAFESTART_RTP_5: [u8; 429] = [
    0x61, 0xE0, 0x00, 0xC0, 0x01, 0xBE, 0x40, 0x8E, 0x0A, 0xFA, 0xD2, 0xAF, 0x5A, 0xFA, 0xD7, 0xD6,
    0xBE, 0xB5, 0xF5, 0xAF, 0xAD, 0x7D, 0x6B, 0xEB, 0x55, 0xCD, 0x8E, 0xC3, 0xE0, 0x98, 0xAF, 0x11,
    0xE2, 0x3C, 0x47, 0x88, 0xEF, 0xAD, 0x45, 0x70, 0x45, 0x46, 0x37, 0x1D, 0xFB, 0xE0, 0xA3, 0x22,
    0xBB, 0x1D, 0x83, 0x77, 0x30, 0xB0, 0x82, 0xD7, 0x41, 0x20, 0xBE, 0x21, 0x71, 0x0B, 0x88, 0x5F,
    0x82, 0x48, 0x24, 0xDB, 0xDD, 0xF4, 0x7A, 0xCE, 0xF1, 0xA7, 0xDE, 0x3D, 0xCD, 0xC6, 0x3A, 0xB5,
    0xBD, 0xF1, 0xAB, 0x6C, 0x42, 0xE2, 0x17, 0x10, 0xB8, 0xF7, 0x6C, 0x75, 0x10, 0xFF, 0x8D, 0x5C,
    0xAA, 0xB9, 0x3B, 0xC6, 0x9F, 0x94, 0xEB, 0x9D, 0xF3, 0xBC, 0x69, 0xF9, 0x4F, 0xE7, 0x7C, 0xEF,
    0x1A, 0x7E, 0x53, 0xF9, 0xDF, 0x3B, 0xC6, 0x9F, 0x94, 0xFE, 0x77, 0xCE, 0xF6, 0x2B, 0xC4, 0x78,
    0x8F, 0x11, 0xE2, 0x3C, 0x47, 0x88, 0xF1, 0x1E, 0x23, 0xC4, 0x79, 0xF9, 0x4F, 0xE7, 0x7C, 0xEF,
    0xF1, 0x9E, 0x34, 0xF0, 0xD7, 0x00, 0x03, 0xA0, 0x1A, 0x64, 0x1B, 0x87, 0xE3, 0x05, 0x89, 0xAF,
    0x79, 0xF7, 0xF8, 0xCF, 0x1D, 0x1F, 0xFB, 0x58, 0x0C, 0xA0, 0x81, 0xC4, 0xF1, 0x8E, 0x42, 0x6C,
    0x0B, 0xF6, 0x59, 0x43, 0x07, 0xFC, 0x67, 0x80, 0x87, 0xC1, 0x3E, 0xD3, 0xE3, 0xC2, 0x29, 0xAB,
    0x01, 0xDA, 0xCC, 0x02, 0x0F, 0x88, 0x5A, 0x87, 0xFC, 0x4B, 0x18, 0x20, 0x5F, 0xF8, 0xCE, 0xB7,
    0x6D, 0x0B, 0xE6, 0x22, 0x1A, 0x44, 0xE1, 0x83, 0x07, 0x52, 0x75, 0x47, 0xDE, 0x10, 0x4D, 0x7C,
    0x18, 0xFE, 0x3B, 0x88, 0x36, 0x8F, 0xA9, 0xBF, 0x1D, 0x2E, 0xDC, 0x18, 0x0E, 0x94, 0x43, 0xFC,
    0x77, 0x04, 0x2D, 0x44, 0x9E, 0x41, 0xC2, 0x3E, 0x2C, 0x61, 0x40, 0x10, 0x0A, 0x80, 0x5A, 0x40,
    0x83, 0x02, 0x7F, 0x84, 0x3B, 0x46, 0x08, 0x23, 0x6C, 0xE6, 0x62, 0x0A, 0x68, 0x9E, 0x18, 0x43,
    0x46, 0x8D, 0x05, 0x43, 0xFC, 0x7F, 0x43, 0x08, 0xF8, 0xEF, 0x5E, 0x63, 0xC7, 0x4B, 0x44, 0x8A,
    0x59, 0x16, 0x66, 0xB6, 0xAC, 0xFF, 0x19, 0xD5, 0x06, 0xB5, 0x16, 0xE2, 0x1D, 0x4B, 0xC3, 0xE7,
    0x90, 0x40, 0x1A, 0xBC, 0x77, 0xDB, 0x00, 0xF6, 0xF9, 0xFB, 0x03, 0xFB, 0xC3, 0xFF, 0xC6, 0x71,
    0x96, 0x49, 0x10, 0x85, 0x68, 0xA1, 0xD2, 0xA5, 0x45, 0xD3, 0xD1, 0x30, 0xDF, 0xC2, 0x11, 0x59,
    0xC4, 0x87, 0xF8, 0xCE, 0x6C, 0xE0, 0xEE, 0x84, 0xB0, 0xC4, 0xEC, 0x0C, 0x86, 0x21, 0x8F, 0xA0,
    0xC0, 0xFB, 0x03, 0xA1, 0x9E, 0xA0, 0x7B, 0x64, 0x7A, 0x7F, 0x89, 0xE4, 0xC8, 0x18, 0x03, 0x98,
    0xB0, 0xA2, 0xFC, 0xFF, 0xC1, 0x47, 0x8C, 0x92, 0xB5, 0xAD, 0x0E, 0x24, 0x97, 0x10, 0xB8, 0x85,
    0xC4, 0x2E, 0x21, 0x7E, 0x6B, 0x1C, 0x91, 0xCE, 0xFF, 0x19, 0xDD, 0xF7, 0x77, 0x77, 0x77, 0x77,
    0x78, 0x8F, 0x11, 0xE2, 0x3C, 0x47, 0x88, 0xF1, 0x1E, 0x23, 0xC4, 0x78, 0x8F, 0x11, 0xE2, 0x3C,
    0x47, 0x88, 0x5C, 0x42, 0xE2, 0x17, 0x10, 0xB8, 0x85, 0xC4, 0x2E, 0x23, 0xC0,
];
pub const SAFESTART_RTP_6: [u8; 141] = [
    0x61, 0xE0, 0x01, 0x00, 0x02, 0x3E, 0x42, 0xE0, 0xD3, 0xAD, 0x50, 0xA7, 0xC4, 0x71, 0x3C, 0x44,
    0xC4, 0x8C, 0x4B, 0x31, 0x23, 0x12, 0xE2, 0x15, 0xE2, 0x17, 0x10, 0xB8, 0x85, 0xC4, 0x2E, 0x21,
    0x71, 0x0B, 0x88, 0x58, 0xD3, 0xF9, 0xFC, 0xFE, 0x7F, 0x3F, 0x9F, 0xCF, 0xE7, 0xF1, 0x0B, 0x8F,
    0x78, 0x93, 0x8F, 0xAB, 0xAB, 0xBD, 0xE2, 0xCF, 0xE7, 0xF3, 0xF9, 0xFC, 0xFE, 0x7F, 0x3F, 0x9F,
    0xCF, 0xE3, 0xDE, 0xBE, 0xC7, 0x63, 0x8B, 0x3F, 0x9F, 0xCF, 0xE7, 0xF3, 0xF9, 0xFC, 0xFE, 0x7F,
    0x3F, 0x9D, 0xE2, 0xCF, 0xE7, 0xF3, 0xF9, 0xFC, 0xFE, 0x7F, 0x3F, 0x9F, 0xCF, 0xE7, 0x78, 0xB3,
    0xF9, 0xFC, 0xFE, 0x7F, 0x3F, 0x9F, 0xCF, 0xE7, 0xF3, 0xF9, 0xDE, 0x2C, 0xFE, 0x7F, 0x3F, 0x9F,
    0xCF, 0xE7, 0xF3, 0xF9, 0xFC, 0xFE, 0x77, 0x84, 0xCF, 0xD0, 0x51, 0x40, 0x75, 0x35, 0x2D, 0xFF,
    0x9C, 0xD1, 0xCD, 0x67, 0x34, 0x73, 0x51, 0x0B, 0x88, 0x5C, 0x42, 0xC2, 0xF0,
];
pub const SAFESTART_RTP_7: [u8; 511] = [
    0x61, 0xE0, 0x01, 0x40, 0x02, 0xBE, 0x41, 0x78, 0x2B, 0xE0, 0x93, 0x15, 0xDC, 0x56, 0xCC, 0x3E,
    0xB9, 0x8F, 0xAB, 0x1F, 0x56, 0x3E, 0xAC, 0x7D, 0x58, 0xFA, 0xB1, 0xF5, 0x63, 0xEA, 0xC5, 0x70,
    0x49, 0x87, 0xA0, 0x5B, 0x58, 0xE2, 0x51, 0x15, 0xC1, 0x16, 0x26, 0x4B, 0x16, 0xB5, 0x07, 0x15,
    0xD2, 0x4B, 0x49, 0x2F, 0xFD, 0x7D, 0xB6, 0xF8, 0x85, 0xC4, 0x2E, 0x21, 0x71, 0x0B, 0x88, 0x5C,
    0x42, 0xE2, 0x17, 0xE0, 0x96, 0x26, 0x48, 0xC4, 0xBA, 0x0C, 0x8A, 0x38, 0xBE, 0x5C, 0x21, 0xE5,
    0x1A, 0xFE, 0xA9, 0x97, 0xAA, 0x65, 0xEA, 0x99, 0x7A, 0xA6, 0x5E, 0x08, 0xEC, 0x1B, 0x06, 0xEE,
    0x76, 0x61, 0x1C, 0x21, 0x7E, 0x7A, 0xF9, 0xD8, 0x9D, 0x89, 0x04, 0xF8, 0x6D, 0xD7, 0xD8, 0xEC,
    0x7F, 0x63, 0xB1, 0xC4, 0x5C, 0x42, 0x7F, 0x81, 0xED, 0x5C, 0x87, 0xB5, 0x70, 0x71, 0xF9, 0x82,
    0xEE, 0xDF, 0xBD, 0xFE, 0xF7, 0xEA, 0x38, 0xFC, 0xC7, 0x78, 0xE3, 0xF3, 0x1D, 0xE3, 0x8F, 0xCC,
    0x77, 0x8E, 0x3F, 0x31, 0xDE, 0xB8, 0xCF, 0x04, 0x0F, 0x94, 0x6D, 0x2A, 0xE7, 0xA3, 0x97, 0x62,
    0xB4, 0xCB, 0x75, 0xD9, 0xFA, 0xDC, 0x40, 0xAB, 0xFE, 0x33, 0xAD, 0x00, 0x10, 0x67, 0xC1, 0xF8,
    0x3C, 0xC5, 0xC5, 0x9B, 0x31, 0xE3, 0xED, 0x22, 0x5F, 0xC7, 0xEF, 0x3F, 0x08, 0x68, 0x1F, 0xF2,
    0x84, 0x35, 0x2C, 0x1E, 0xFB, 0xFC, 0x67, 0x49, 0x06, 0x05, 0x35, 0x70, 0x7B, 0x8E, 0x27, 0x7F,
    0x3B, 0xF8, 0x86, 0x46, 0xFF, 0x4F, 0xCD, 0x62, 0x4A, 0x76, 0x0A, 0xCF, 0x3B, 0x60, 0x73, 0x48,
    0x30, 0x5F, 0xC6, 0x7D, 0x05, 0x47, 0x8A, 0x79, 0x15, 0xA5, 0xEE, 0x94, 0x90, 0x4E, 0x57, 0x95,
    0xD6, 0x01, 0x8F, 0xD6, 0xE5, 0xBE, 0xB3, 0x9A, 0x03, 0xE3, 0xFF, 0xE3, 0x3B, 0x52, 0x82, 0x0F,
    0xAE, 0xE7, 0x88, 0xFF, 0x25, 0xDE, 0xAA, 0x60, 0xB7, 0x01, 0x7F, 0x65, 0x56, 0x05, 0x8B, 0xFE,
    0x62, 0xD8, 0x07, 0x15, 0xFD, 0x01, 0xFF, 0x91, 0x7F, 0xE1, 0x1E, 0x11, 0x60, 0xB7, 0x56, 0x03,
    0x14, 0x86, 0x97, 0x7E, 0x90, 0x5F, 0x7E, 0x27, 0xC4, 0x35, 0x5C, 0x75, 0x6C, 0x2A, 0xDD, 0xF8,
    0x7D, 0xFF, 0x2F, 0xFF, 0x19, 0xC0, 0x95, 0xAE, 0xC3, 0x77, 0xD5, 0x4B, 0x3F, 0xE9, 0xC1, 0xBF,
    0xF3, 0x7C, 0xA0, 0x2F, 0xEE, 0x03, 0x7A, 0x34, 0xBB, 0x68, 0xFE, 0x6D, 0x7E, 0x0B, 0x17, 0xE4,
    0x36, 0x34, 0xF8, 0xAB, 0xAF, 0xAF, 0xF8, 0xCE, 0x30, 0x3F, 0x76, 0x3E, 0x3F, 0xD1, 0x51, 0xC2,
    0x16, 0x47, 0xF8, 0xEF, 0xF3, 0x5E, 0x26, 0xE5, 0xFE, 0xB9, 0xCF, 0x7F, 0x75, 0x76, 0xF0, 0x0E,
    0xDF, 0xCB, 0xF9, 0xF8, 0x95, 0x1C, 0x6E, 0xFF, 0xFC, 0x67, 0x46, 0x82, 0xD8, 0x69, 0xC7, 0x76,
    0x32, 0x03, 0xC5, 0xDA, 0x3D, 0x15, 0xF5, 0x5A, 0x5B, 0xE7, 0xF3, 0x7E, 0xBE, 0x13, 0xCE, 0x84,
    0xBF, 0x8C, 0xE5, 0x98, 0x25, 0x7C, 0x0D, 0x70, 0x38, 0x8D, 0xB8, 0x76, 0x59, 0x4B, 0xBE, 0x9C,
    0xCF, 0x68, 0x0F, 0xF4, 0x1E, 0xE4, 0x5E, 0xC9, 0x0F, 0x81, 0x9F, 0x3F, 0xE3, 0x3A, 0xC3, 0x96,
    0x27, 0x82, 0x26, 0xC0, 0xF6, 0x60, 0x25, 0x7B, 0xC0, 0x97, 0xEE, 0x5F, 0xD8, 0xF5, 0xA8, 0xC3,
    0x4A, 0xF9, 0x6F, 0x1F, 0xCC, 0x5D, 0x83, 0x75, 0xFE, 0x27, 0xD1, 0xE5, 0xD7, 0xF7, 0x10, 0x4E,
    0xBE, 0xB8, 0x63, 0x2A, 0xF2, 0xD3, 0x03, 0xBC, 0xD0, 0x1D, 0xD3, 0x44, 0xC7, 0xC5, 0x73, 0x88,
    0x1C, 0x20, 0xB3, 0x2C, 0xFE, 0xC2, 0x61, 0x44, 0x2E, 0x21, 0x71, 0x0B, 0xF0, 0x5B, 0x06, 0xBA,
    0x67, 0x2A, 0xE9, 0x9F, 0x03, 0xB4, 0xD0, 0x1D, 0xA6, 0x8C, 0xC0, 0x28, 0x21, 0x78, 0x8F,
];
pub const SAFESTART_RTP_8: [u8; 73] = [
    0x61, 0xE0, 0x01, 0x80, 0x03, 0x3E, 0x40, 0xBE, 0x0D, 0x39, 0xF9, 0x8F, 0xB6, 0xDB, 0x4D, 0x0C,
    0x08, 0x28, 0xAF, 0x88, 0xB4, 0x07, 0x11, 0x59, 0x61, 0xC4, 0x56, 0x5F, 0xB6, 0x2A, 0xF8, 0x85,
    0x84, 0xCF, 0xC5, 0xE9, 0x6B, 0x10, 0xB8, 0x85, 0xAE, 0x09, 0xED, 0xDB, 0xB7, 0x6F, 0xA9, 0x4F,
    0xC2, 0x87, 0xE1, 0x43, 0xF0, 0xA1, 0xF8, 0x50, 0xFC, 0x3E, 0x10, 0x5D, 0xCE, 0x92, 0x5B, 0x09,
    0x85, 0x18, 0xB4, 0xAA, 0x21, 0x71, 0x0B, 0x0B, 0xC0,
];
pub const SAFESTART_RTP_9: [u8; 35] = [
    0x61, 0xE0, 0x01, 0xC0, 0x03, 0xBE, 0x40, 0x47, 0x81, 0xC8, 0x7B, 0x93, 0xCA, 0x0F, 0x50, 0x3B,
    0xC1, 0xEA, 0x0F, 0x7F, 0x85, 0x0E, 0xF0, 0xA1, 0xDE, 0x14, 0x3B, 0xC2, 0x87, 0x78, 0x4C, 0xFC,
    0xE7, 0x78, 0x56,
];
pub const SAFESTART_RTP_10: [u8; 173] = [
    0x61, 0xE0, 0x02, 0x00, 0x04, 0x3E, 0x40, 0x5F, 0xF8, 0xCB, 0xBB, 0xBB, 0xBB, 0xBB, 0xBB, 0xBB,
    0xBB, 0xBC, 0x4F, 0x89, 0xF1, 0x3E, 0x27, 0xC4, 0xF8, 0x9F, 0x13, 0xE2, 0x7C, 0x4F, 0x89, 0xF1,
    0x3E, 0x27, 0xC4, 0xF8, 0x9F, 0x13, 0xE2, 0x7C, 0x4F, 0x89, 0xF1, 0x3E, 0x7F, 0x3F, 0x9F, 0xCF,
    0xE7, 0xF3, 0xF9, 0xFC, 0xFE, 0x7F, 0x3F, 0x9F, 0xCF, 0xE7, 0xF3, 0xF9, 0xFC, 0xFE, 0x7F, 0x3F,
    0x9F, 0xCF, 0xE7, 0xF3, 0xF1, 0x87, 0x7C, 0xFE, 0x7F, 0x3F, 0x9F, 0xCF, 0xE7, 0xF3, 0xF9, 0xF8,
    0x4C, 0xFE, 0x7F, 0x10, 0xB0, 0x91, 0xFC, 0xFE, 0x7E, 0x2F, 0x89, 0xB6, 0x65, 0x2D, 0xB9, 0x4B,
    0x6E, 0x52, 0xDB, 0x94, 0x9C, 0xFE, 0x7F, 0x3F, 0x16, 0x3D, 0xD8, 0xD3, 0xEB, 0x52, 0xBE, 0x57,
    0xCE, 0x7F, 0x3F, 0x9F, 0x8B, 0x3B, 0xB1, 0x9C, 0xFE, 0x7F, 0x3F, 0x16, 0x77, 0x63, 0x39, 0xFC,
    0xFE, 0x7E, 0x53, 0xF3, 0x1D, 0xD8, 0xC2, 0x67, 0xE2, 0x38, 0xCB, 0xBB, 0xBB, 0xDE, 0xEE, 0xEE,
    0xF7, 0xF8, 0x8A, 0xAE, 0xAB, 0xC4, 0x78, 0x8F, 0x11, 0xE2, 0x3C, 0x47, 0x88, 0xF1, 0x1E, 0x23,
    0xC4, 0x78, 0x8F, 0x11, 0xE2, 0x17, 0x10, 0xB8, 0x85, 0xC4, 0x2E, 0x21, 0x6E,
];

pub const FRAGMENT_SIZE: usize = 1400;

/// Save encoded stream to RTP dump
pub fn save_rtp_file(rtp_filename: String, rtp_nal: &Vec<Vec<u8>>, enable_safestart: bool) {
    println!("   Writing RTP output: {}", &rtp_filename);

    // Stage 1: NAL to packet (single packet mode for now)

    let mut packets: Vec<Vec<u8>> = Vec::new();
    let mut rtp_nal_mod: Vec<Vec<u8>> = Vec::new();
    let mut seq_num: u16 = 0x1234;
    let mut timestamp: u32 = 0x11223344;
    let ssrc: u32 = 0x77777777;
    if enable_safestart {
        rtp_nal_mod.push(SAFESTART_RTP_0.to_vec());
        rtp_nal_mod.push(SAFESTART_RTP_1.to_vec());
        rtp_nal_mod.push(SAFESTART_RTP_2.to_vec());
        rtp_nal_mod.push(SAFESTART_RTP_3.to_vec());
        rtp_nal_mod.push(SAFESTART_RTP_4.to_vec());
        rtp_nal_mod.push(SAFESTART_RTP_5.to_vec());
        rtp_nal_mod.push(SAFESTART_RTP_6.to_vec());
        rtp_nal_mod.push(SAFESTART_RTP_7.to_vec());
        rtp_nal_mod.push(SAFESTART_RTP_8.to_vec());
        rtp_nal_mod.push(SAFESTART_RTP_9.to_vec());
        rtp_nal_mod.push(SAFESTART_RTP_10.to_vec());
    }

    rtp_nal_mod.extend(rtp_nal.clone());
    for i in 0..rtp_nal_mod.len() {
        let header_byte = 0x80; // version 2, no padding, no extensions, no CSRC, marker = false;
        let nal_type = rtp_nal_mod[i][0] & 0x1f;

        packets.push(Vec::new());
        packets[i].push(header_byte);
        let mut payload_type = 104; // common H.264
        if nal_type == 5 {
            payload_type = payload_type + 0x80; // add marker
        }
        if nal_type == 1 {
            payload_type = payload_type + 0x80; // add marker
            timestamp += 3000;
        }
        if nal_type == 28 {
            let inner_nal_type = rtp_nal_mod[i][1] & 0x1f;
            if inner_nal_type == 5 {
                payload_type = payload_type + 0x80; // add marker
            }
            if inner_nal_type == 1 {
                payload_type = payload_type + 0x80; // add marker
                if (rtp_nal_mod[i][1] & 0x80) != 0 {
                    timestamp += 3000;
                }
            }
        }
        packets[i].push(payload_type);
        packets[i].extend(seq_num.to_be_bytes());
        seq_num += 1;
        packets[i].extend(timestamp.to_be_bytes());
        packets[i].extend(ssrc.to_be_bytes());
        packets[i].extend(rtp_nal_mod[i].iter());
    }

    let mut out_bytes: Vec<u8> = Vec::new();
    let s = "#!rtpplay1.0 127.0.0.1/48888\n";
    let header = s.bytes();
    out_bytes.extend(header);

    let start_sec: u32 = 0;
    let start_usec: u32 = 0;
    let source: u32 = 0;
    let port: u16 = 0;
    let padding: u16 = 0;

    out_bytes.extend(start_sec.to_be_bytes());
    out_bytes.extend(start_usec.to_be_bytes());
    out_bytes.extend(source.to_be_bytes());
    out_bytes.extend(port.to_be_bytes());
    out_bytes.extend(padding.to_be_bytes());

    for i in 0..packets.len() {
        let plen: u16 = packets[i].len().try_into().unwrap();
        let blen: u16 = plen + 8;
        let ts: u32 = 0;

        out_bytes.extend(blen.to_be_bytes());
        out_bytes.extend(plen.to_be_bytes());
        out_bytes.extend(ts.to_be_bytes());
        out_bytes.extend(packets[i].iter());
    }

    let mut f = match File::create(&rtp_filename) {
        Err(_) => panic!("couldn't open {}", &rtp_filename),
        Ok(file) => file,
    };

    match f.write_all(out_bytes.as_slice()) {
        Err(_) => panic!("couldn't write to file {}", &rtp_filename),
        Ok(()) => (),
    };
}


/// Encode a Single-Time Aggregation Unit without DON (STAP-A)
///
///   0                   1                   2                   3
///   0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
///   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///   |                          RTP Header                           |
///   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///   |STAP-A NAL HDR |         NALU 1 Size           | NALU 1 HDR    |
///   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///   |                         NALU 1 Data                           |
///   :                                                               :
///   +               +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///   |               | NALU 2 Size                   | NALU 2 HDR    |
///   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///   |                         NALU 2 Data                           |
///   :                                                               :
///   |                               +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///   |                               :...OPTIONAL RTP padding        |
///   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///
///   Figure 7.  An example of an RTP packet including an STAP-A
///              containing two single-time aggregation units
#[allow(dead_code)]
pub fn encode_stap_a(p : StapA) -> Vec<u8> {
    let mut res = Vec::new();
    for i in 0..p.nalus.len() {
        res.extend(generate_fixed_length_value(p.nalu_sizes[i] as u32, 16));
        res.extend(p.nalus[i].content.iter());
    }

    return res;
}

/// Encode a Single-Time Aggregation Unit with DON (STAP-B)
///
///   0                   1                   2                   3
///   0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
///   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///   |                          RTP Header                           |
///   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///   |STAP-B NAL HDR | DON                           | NALU 1 Size   |
///   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///   | NALU 1 Size   | NALU 1 HDR    | NALU 1 Data                   |
///   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+                               +
///   :                                                               :
///   +               +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///   |               | NALU 2 Size                   | NALU 2 HDR    |
///   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///   |                       NALU 2 Data                             |
///   :                                                               :
///   |                               +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///   |                               :...OPTIONAL RTP padding        |
///   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
#[allow(dead_code)]
pub fn encode_stap_b(_p : StapB) {
    // Encode a Decoding Order Number (DON) 16 bits long
    // while more_data() {
    //   Encode a NAL unit size that is 16 bits
    //   Encode a NALU of set size
    // }
}


/// Encode a Multi-Time Aggregation Packet (MTAP) with 16-bit timestamp offset (TS)
///
///   0                   1                   2                   3
///   0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
///   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///   |                          RTP Header                           |
///   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///   |MTAP16 NAL HDR |  decoding order number base   | NALU 1 Size   |
///   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///   |  NALU 1 Size  |  NALU 1 DOND  |       NALU 1 TS offset        |
///   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///   |  NALU 1 HDR   |  NALU 1 DATA                                  |
///   +-+-+-+-+-+-+-+-+                                               +
///   :                                                               :
///   +               +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///   |               | NALU 2 SIZE                   |  NALU 2 DOND  |
///   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///   |       NALU 2 TS offset        |  NALU 2 HDR   |  NALU 2 DATA  |
///   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+               |
///   :                                                               :
///   |                               +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///   |                               :...OPTIONAL RTP padding        |
///   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
#[allow(dead_code)]
pub fn encode_mtap16(_p : Mtap16) {
    // While more_data() {
    //   Encode a NALU Size that is 16 bits
    //   Encode a Decoding Order Number Difference (DOND) that is 8-bits
    //   Encode a 16-bit Timestamp Offset
    //   Encode a NALU of nalu size
    // }
}

/// Encode a Multi-Time Aggregation Packet (MTAP) with 24-bit timestamp offset (TS)
///
///   0                   1                   2                   3
///   0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
///   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///   |                          RTP Header                           |
///   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///   |MTAP24 NAL HDR |  decoding order number base   | NALU 1 Size   |
///   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///   |  NALU 1 Size  |  NALU 1 DOND  |       NALU 1 TS offs          |
///   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///   |NALU 1 TS offs |  NALU 1 HDR   |  NALU 1 DATA                  |
///   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+                               +
///   :                                                               :
///   +               +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///   |               | NALU 2 SIZE                   |  NALU 2 DOND  |
///   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///   |       NALU 2 TS offset                        |  NALU 2 HDR   |
///   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///   |  NALU 2 DATA                                                  |
///   :                                                               :
///   |                               +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///   |                               :...OPTIONAL RTP padding        |
///   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
#[allow(dead_code)]
pub fn encode_mtap24(_p : Mtap24) {
    // While more_data() {
    //   Encode a NALU Size that is 16 bits
    //   Encode a Decoding Order Number Difference (DOND) that is 8-bits
    //   Encode a 24-bit Timestamp Offset
    //   Encode a NALU of nalu size
    // }
}

/// Encapsulate a Fragmentation Unit (FU) without a DON (FU-A)
///   0                   1
///   0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6
///   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///   | FU indicator  |   FU header   |
///   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
pub fn encapsulate_fu_a(nal : &Vec<u8>, nh : &NALUheader) -> Vec<Vec<u8>> {
    let mut res = Vec::new();

    // 28 is FU-A
    let fu_indicator: u8 = 28 | nh.forbidden_zero_bit << 7 | (nh.nal_ref_idc << 5);
    let fua_chunks = nal.chunks(FRAGMENT_SIZE);
    let last_idx = fua_chunks.len() - 1;
    for (i, chunk) in fua_chunks.enumerate() {
        let mut fua_bytes: Vec<u8> = Vec::new();
        fua_bytes.push(fu_indicator);

        // Encode FU header
        // +---------------+
        // |0|1|2|3|4|5|6|7|
        // +-+-+-+-+-+-+-+-+
        // |S|E|R|  Type   |
        // +---------------+
        //  S: Start bit indicating the start of an FU
        //  E: End bit indicating the end of an FU
        //  R: Reserved, please set to 0
        //  Type: NALU Payload type
        let mut fu_header = nh.nal_unit_type;
        if i == 0 {
            fu_header |= 0x80; // S = 1
        }
        if i == last_idx {
            fu_header |= 0x40; // E = 1
        }
        fua_bytes.push(fu_header);
        fua_bytes.extend(chunk);
        res.push(fua_bytes);
    }

    // TODO: allow empty FUs

    return res;
}

/// Encodes a Fragmentation Unit (FU) with a DON (FU-B)
///
///   0                   1                   2                   3
///   0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
///   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///   | FU indicator  |   FU header   |               DON             |
///   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-|
///   |                                                               |
///   |                         FU payload                            |
///   |                                                               |
///   |                               +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///   |                               :...OPTIONAL RTP padding        |
///   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///
/// NOTE: uses the same DON for each FU atm
#[allow(dead_code)]
pub fn encapsulate_fu_b(nal : &Vec<u8>, nh : &NALUheader, don : u16) -> Vec<Vec<u8>> {
    let mut res = Vec::new();

    // 29 is FU-B
    let fu_indicator: u8 = 29 | nh.forbidden_zero_bit << 7 | (nh.nal_ref_idc << 5);
    let encoded_don = generate_fixed_length_value(don as u32, 16);
    let fub_chunks = nal.chunks(FRAGMENT_SIZE);
    let last_idx = fub_chunks.len() - 1;
    for (i, chunk) in fub_chunks.enumerate() {
        let mut fub_bytes: Vec<u8> = Vec::new();
        fub_bytes.push(fu_indicator);

        // Encode FU header
        // +---------------+
        // |0|1|2|3|4|5|6|7|
        // +-+-+-+-+-+-+-+-+
        // |S|E|R|  Type   |
        // +---------------+
        //  S: Start bit indicating the start of an FU
        //  E: End bit indicating the end of an FU
        //  R: Reserved, please set to 0
        //  Type: NALU Payload type
        let mut fu_header = nh.nal_unit_type;
        if i == 0 {
            fu_header |= 0x80; // S = 1
        }
        if i == last_idx {
            fu_header |= 0x40; // E = 1
        }
        fub_bytes.push(fu_header);
        // Encode DON
        fub_bytes.extend(encoded_don.iter());
        // Add the rest of the payload
        fub_bytes.extend(chunk);
        res.push(fub_bytes);
    }

    // TODO: allow empty FUs

    return res;
}
