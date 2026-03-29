export type FlagType = 'bool' | 'int' | 'string'

export function detectType(value: string): FlagType {
    if (value === 'true' || value === 'false') return 'bool'
    if (/^-?\d+$/.test(value)) return 'int'
    return 'string'
}

export function validateValue(value: string, type: FlagType): boolean {
    if (type === 'bool') return value === 'true' || value === 'false'
    if (type === 'int') return /^-?\d+$/.test(value)
    return true
}
