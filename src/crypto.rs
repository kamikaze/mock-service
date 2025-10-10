use openssl::rsa::{Padding, Rsa};
use openssl::symm::{Cipher, decrypt};
use std::error::Error;

fn decrypt_aes_key(
    encrypted_key: &[u8],
    private_key_pem: &[u8],
) -> Result<Vec<u8>, Box<dyn Error>> {
    let rsa = Rsa::private_key_from_pem(private_key_pem)?;
    let mut decrypted = vec![0; rsa.size() as usize];
    let len = rsa.private_decrypt(encrypted_key, &mut decrypted, Padding::PKCS1)?;
    decrypted.truncate(len);
    Ok(decrypted)
}

pub fn decrypt_data_with_aes(
    encrypted_data: &[u8],
    encrypted_aes_key: &[u8],
    private_key_pem: &[u8],
) -> Result<Vec<u8>, Box<dyn Error>> {
    // Decrypt AES key using RSA private key
    let aes_key = decrypt_aes_key(encrypted_aes_key, private_key_pem)?;

    // Decode base64 AES key (this is the password for OpenSSL)
    let aes_password = std::str::from_utf8(&aes_key)?.trim();

    // OpenSSL format: "Salted__" + 8 bytes salt + encrypted data
    if encrypted_data.len() < 16 || &encrypted_data[0..8] != b"Salted__" {
        return Err("Invalid encrypted file format".into());
    }

    let salt = &encrypted_data[8..16];
    let ciphertext = &encrypted_data[16..];

    // Derive key and IV using OpenSSL's EVP_BytesToKey (equivalent to -pbkdf2)
    let mut key = [0u8; 32];
    let mut iv = [0u8; 16];
    openssl::pkcs5::pbkdf2_hmac(
        aes_password.as_bytes(),
        salt,
        10000,
        openssl::hash::MessageDigest::sha256(),
        &mut key,
    )?;
    openssl::pkcs5::pbkdf2_hmac(
        &[key.as_slice(), aes_password.as_bytes()].concat(),
        salt,
        1,
        openssl::hash::MessageDigest::sha256(),
        &mut iv,
    )?;

    // Decrypt using AES-256-CBC
    let decrypted = decrypt(Cipher::aes_256_cbc(), &key, Some(&iv), ciphertext)?;

    Ok(decrypted)
}
