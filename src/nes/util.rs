use bincode::Decode;
use bincode::Encode;
use image::RgbaImage;

pub struct BincodeImage {
    pub image: RgbaImage,
}

impl std::ops::Deref for BincodeImage {
    type Target = RgbaImage;

    fn deref(&self) -> &Self::Target {
        &self.image
    }
}

impl Encode for BincodeImage {
    fn encode<E: bincode::enc::Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), bincode::error::EncodeError> {
        bincode::Encode::encode(&self.image.width(), encoder)?;
        bincode::Encode::encode(&self.image.height(), encoder)?;
        bincode::Encode::encode(&self.image.as_raw(), encoder)?;
        Ok(())
    }
}

impl Decode for BincodeImage {
    fn decode<D: bincode::de::Decoder>(
        decoder: &mut D,
    ) -> Result<Self, bincode::error::DecodeError> {
        Ok(BincodeImage {
            image: RgbaImage::from_raw(
                Decode::decode(decoder)?,
                Decode::decode(decoder)?,
                Decode::decode(decoder)?,
            )
            .unwrap(),
        })
    }
}
