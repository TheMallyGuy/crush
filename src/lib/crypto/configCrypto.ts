const PBKDF2_ITERATIONS = 200_000
const SALT_BYTES = 16
const IV_BYTES = 12
const ENVELOPE_VERSION = 1

type Envelope = {
    v: number
    salt: string
    iv: string
    data: string
}

export class WrongPasswordError extends Error {
    constructor(message = 'wrong password') {
        super(message)
        this.name = 'WrongPasswordError'
    }
}

function toBase64(bytes: Uint8Array): string {
    let binary = ''
    for (const b of bytes) binary += String.fromCharCode(b)
    return btoa(binary)
}

function fromBase64(b64: string): Uint8Array {
    const binary = atob(b64)
    const bytes = new Uint8Array(binary.length)
    for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i)
    return bytes
}

async function deriveKey(
    password: string,
    salt: Uint8Array
): Promise<CryptoKey> {
    const keyMaterial = await crypto.subtle.importKey(
        'raw',
        new TextEncoder().encode(password),
        'PBKDF2',
        false,
        ['deriveKey']
    )
    return crypto.subtle.deriveKey(
        {
            name: 'PBKDF2',
            salt: salt as BufferSource,
            iterations: PBKDF2_ITERATIONS,
            hash: 'SHA-256',
        },
        keyMaterial,
        { name: 'AES-GCM', length: 256 },
        false,
        ['encrypt', 'decrypt']
    )
}

export async function encryptConfig(
    plaintext: string,
    password: string
): Promise<string> {
    const salt = crypto.getRandomValues(new Uint8Array(SALT_BYTES))
    const iv = crypto.getRandomValues(new Uint8Array(IV_BYTES))
    const key = await deriveKey(password, salt)

    const ciphertext = await crypto.subtle.encrypt(
        { name: 'AES-GCM', iv },
        key,
        new TextEncoder().encode(plaintext)
    )

    const envelope: Envelope = {
        v: ENVELOPE_VERSION,
        salt: toBase64(salt),
        iv: toBase64(iv),
        data: toBase64(new Uint8Array(ciphertext)),
    }
    return JSON.stringify(envelope)
}

export async function decryptConfig(
    envelopeJson: string,
    password: string
): Promise<string> {
    let envelope: Envelope
    try {
        envelope = JSON.parse(envelopeJson) as Envelope
    } catch {
        throw new WrongPasswordError('cloud data is not encrypted or is corrupted')
    }

    if (
        envelope?.v !== ENVELOPE_VERSION ||
        !envelope.salt ||
        !envelope.iv ||
        !envelope.data
    ) {
        throw new WrongPasswordError('cloud data is not encrypted or is corrupted')
    }

    const key = await deriveKey(password, fromBase64(envelope.salt))

    let plaintext: ArrayBuffer
    try {
        plaintext = await crypto.subtle.decrypt(
            { name: 'AES-GCM', iv: fromBase64(envelope.iv) as BufferSource },
            key,
            fromBase64(envelope.data) as BufferSource
        )
    } catch {
        throw new WrongPasswordError()
    }

    return new TextDecoder().decode(plaintext)
}
