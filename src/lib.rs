
const QUANT_MASK: u8  = 0xf;  // 15,  0000 1111
const BIAS:       u8  = 0x84; // 132, 1000 0100
const SEG_MASK:   u8  = 0x70; // 112, 0111 0000
const SEG_SHIFT:  u8  = 4;
const SIGN_BIT:   u8  = 0x80; // 128, 1000 0000
const CLIP:       i16 = 8159; // a little less than 2^13=8192, ah its 2^13 - 8159 = 33. 

const SEG_UEND_SIZE: usize = 8;
const SEG_UEND: [i16; SEG_UEND_SIZE] = [0x3F, 0x7F, 0xFF, 0x1FF, 0x3FF, 0x7FF, 0xFFF, 0x1FFF];


fn search(val: i16, table: [i16; SEG_UEND_SIZE], size: usize) -> u8 {
    let mut count: u8 = 0;
    for x in table.iter() {
        if val <= *x {
            return count;
        }
        count += 1;
    }
    return size as u8;
}


/// ulaw2linear takes an unsigned char in c++
///  unsigned char is 0 to 255 or 8 bits
///  returns a short: 16 bits, signed.
fn ulaw2linear(mut u_val: u8) -> i16 {
    
    u_val = !u_val;

    let mut t: i16 = (((u_val & QUANT_MASK) << 3) + BIAS) as i16;

    let shiftval = (u_val & SEG_MASK) >> SEG_SHIFT;
    t = t << shiftval;
    
    if u_val & SIGN_BIT > 0 {
        t = BIAS as i16 - t;
    } else {
        t = t - BIAS as i16;
    }
    return t;
}


fn linear2ulaw(mut pcm_val: i16) -> u8  {
    pcm_val = pcm_val >> 2;
    let mut mask: u8 = 0;
    
    if pcm_val < 0 {
        pcm_val = -pcm_val;
        mask = 0x7F; // mask everything except sign bit
    } else {
        mask = 0xFF; // mask everything
    }

    if pcm_val > CLIP {
        pcm_val = CLIP; // CLIP it!
    }

    pcm_val += (BIAS >> 2) as i16; // u8 loses 2 digits

    /* Convert the scaled magnitute to segment number. */
    let seg: u8 = crate::search(pcm_val, SEG_UEND, SEG_UEND_SIZE);

    if seg >= 8 {
        return 0x7F ^ mask;
    } else {
        //     u8/a value 0-8 << 4 | i16 >> up to 7, then only last 4 digits pass through
        let uval: u8 = (seg << 4) | (((pcm_val >> (seg + 1)) as u8) & 0xf);
        return uval ^ mask;
    }
}


#[cfg(test)]
mod tests {
    use std::collections::VecDeque;
    
    fn encodeDecode(testVal: i16) -> i16 {
        let encoded: u8 = crate::linear2ulaw(testVal);
        let decoded = crate::ulaw2linear(encoded);
        return decoded;
    }
    
    #[test]
    fn it_works() {
        assert_eq!(-31100, crate::ulaw2linear(1));
        assert_ne!(-31101, crate::ulaw2linear(1));
        let mut results = VecDeque::new();
        for i in -8031..8031 {
            let decoded = encodeDecode(i);
            let lastPair = results.back();
            match lastPair {
                None => results.push_back((i, decoded)),
                Some(pair) => {
                    if pair.1 != decoded {
                        results.push_back((i, decoded));
                    } 
                }
            }
        }
        println!("{:?}", results);

        assert_eq!(results.len(), KNOWN_PAIRS.len());
        
        for (i, el) in KNOWN_PAIRS.iter().enumerate() {
            assert_eq!(el.0, results[i].0);
            assert_eq!(el.1, results[i].1);
        }
    }

    // test data
    //const AMPLITUDE_RESOLUTION_SIZE: u32 = 2 ^ 14; // 14 bit space
    const AMPLITUDE_RESOLUTION_SIZE: usize = 191;
    const KNOWN_PAIRS: [(i16, i16); AMPLITUDE_RESOLUTION_SIZE] = [(-8031, -7932),(-7800, -7676),(-7544, -7420),(-7288, -7164),(-7032, -6908),(-6776, -6652),(-6520, -6396),(-6264, -6140),(-6008, -5884),(-5752, -5628),(-5496, -5372),(-5240, -5116),(-4984, -4860),(-4728, -4604),(-4472, -4348),(-4216, -4092),(-3960, -3900),(-3832, -3772),(-3704, -3644),(-3576, -3516),(-3448, -3388),(-3320, -3260),(-3192, -3132),(-3064, -3004),(-2936, -2876),(-2808, -2748),(-2680, -2620),(-2552, -2492),(-2424, -2364),(-2296, -2236),(-2168, -2108),(-2040, -1980),(-1912, -1884),(-1848, -1820),(-1784, -1756),(-1720, -1692),(-1656, -1628),(-1592, -1564),(-1528, -1500),(-1464, -1436),(-1400, -1372),(-1336, -1308),(-1272, -1244),(-1208, -1180),(-1144, -1116),(-1080, -1052),(-1016, -988),(-952, -924),(-888, -876),(-856, -844),(-824, -812),(-792, -780),(-760, -748),(-728, -716),(-696, -684),(-664, -652),(-632, -620),(-600, -588),(-568, -556),(-536, -524),(-504, -492),(-472, -460),(-440, -428),(-408, -396),(-376, -372),(-360, -356),(-344, -340),(-328, -324),(-312, -308),(-296, -292),(-280, -276),(-264, -260),(-248, -244),(-232, -228),(-216, -212),(-200, -196),(-184, -180),(-168, -164),(-152, -148),(-136, -132),(-120, -120),(-112, -112),(-104, -104),(-96, -96),(-88, -88),(-80, -80),(-72, -72),(-64, -64),(-56, -56),(-48, -48),(-40, -40),(-32, -32),(-24, -24),(-16, -16),(-8, -8),(0, 0),(4, 8),(12, 16),(20, 24),(28, 32),(36, 40),(44, 48),(52, 56),(60, 64),(68, 72),(76, 80),(84, 88),(92, 96),(100, 104),(108, 112),(116, 120),(124, 132),(140, 148),(156, 164),(172, 180),(188, 196),(204, 212),(220, 228),(236, 244),(252, 260),(268, 276),(284, 292),(300, 308),(316, 324),(332, 340),(348, 356),(364, 372),(380, 396),(412, 428),(444, 460),(476, 492),(508, 524),(540, 556),(572, 588),(604, 620),(636, 652),(668, 684),(700, 716),(732, 748),(764, 780),(796, 812),(828, 844),(860, 876),(892, 924),(956, 988),(1020, 1052),(1084, 1116),(1148, 1180),(1212, 1244),(1276, 1308),(1340, 1372),(1404, 1436),(1468, 1500),(1532, 1564),(1596, 1628),(1660, 1692),(1724, 1756),(1788, 1820),(1852, 1884),(1916, 1980),(2044, 2108),(2172, 2236),(2300, 2364),(2428, 2492),(2556, 2620),(2684, 2748),(2812, 2876),(2940, 3004),(3068, 3132),(3196, 3260),(3324, 3388),(3452, 3516),(3580, 3644),(3708, 3772),(3836, 3900),(3964, 4092),(4220, 4348),(4476, 4604),(4732, 4860),(4988, 5116),(5244, 5372),(5500, 5628),(5756, 5884),(6012, 6140),(6268, 6396),(6524, 6652),(6780, 6908),(7036, 7164),(7292, 7420),(7548, 7676),(7804, 7932)];
}
