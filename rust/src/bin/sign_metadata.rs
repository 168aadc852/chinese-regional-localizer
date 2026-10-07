use base64::{engine::general_purpose::STANDARD, Engine as _};
use chinese_regional_localizer::{sign_detached, SignedPayloadKind, TrustedKey};
use ed25519_dalek::SigningKey;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

fn value_after(args: &[String], flag: &str) -> Option<String> {
    args.windows(2)
        .find(|pair| pair[0] == flag)
        .map(|pair| pair[1].clone())
}

fn fail(message: impl AsRef<str>) -> ExitCode {
    eprintln!("{}", message.as_ref());
    ExitCode::from(2)
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    let Some(kind_raw) = value_after(&args, "--kind") else {
        return fail("missing --kind (package|catalog)");
    };
    let kind = match kind_raw.as_str() {
        "package" => SignedPayloadKind::PackageManifest,
        "catalog" => SignedPayloadKind::ReleaseCatalog,
        _ => return fail("--kind must be package or catalog"),
    };
    let Some(input) = value_after(&args, "--input").map(PathBuf::from) else {
        return fail("missing --input");
    };
    let Some(secret_key_file) = value_after(&args, "--secret-key-file").map(PathBuf::from) else {
        return fail("missing --secret-key-file");
    };
    let Some(output) = value_after(&args, "--output").map(PathBuf::from) else {
        return fail("missing --output");
    };

    let payload = match fs::read(&input) {
        Ok(value) => value,
        Err(error) => return fail(format!("failed to read input: {error}")),
    };
    let secret_text = match fs::read_to_string(&secret_key_file) {
        Ok(value) => value,
        Err(error) => return fail(format!("failed to read secret key file: {error}")),
    };
    let decoded = match STANDARD.decode(secret_text.trim()) {
        Ok(value) => value,
        Err(_) => return fail("secret key file must contain base64 for exactly 32 seed bytes"),
    };
    let secret: [u8; 32] = match decoded.try_into() {
        Ok(value) => value,
        Err(_) => return fail("secret key file must contain base64 for exactly 32 seed bytes"),
    };
    let signing_key = SigningKey::from_bytes(&secret);
    let envelope = sign_detached(kind, &payload, &signing_key);
    let serialized = match serde_json::to_vec_pretty(&envelope) {
        Ok(mut value) => {
            value.push(b'\n');
            value
        }
        Err(error) => return fail(format!("failed to serialize signature: {error}")),
    };
    if let Err(error) = fs::write(&output, serialized) {
        return fail(format!("failed to write signature: {error}"));
    }

    if let Some(public_output) = value_after(&args, "--public-key-output").map(PathBuf::from) {
        let trusted_key = TrustedKey::from_verifying_key(&signing_key.verifying_key());
        let public_json = match serde_json::to_vec_pretty(&trusted_key) {
            Ok(mut value) => {
                value.push(b'\n');
                value
            }
            Err(error) => return fail(format!("failed to serialize public key: {error}")),
        };
        if let Err(error) = fs::write(&public_output, public_json) {
            return fail(format!("failed to write public key: {error}"));
        }
    }

    println!("signed with {}", envelope.key_id);
    ExitCode::SUCCESS
}
