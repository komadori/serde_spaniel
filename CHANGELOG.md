# Changelog

## Serde Spaniel 0.4.0 (2022-08-02)

### Changed
- Updated RustyLine dependency to 10.0.

## Serde Spaniel 0.3.0 (2021-05-24)

### Added
- Added example program.

### Changed
- Updated RustyLine dependency to 8.2.
- Changed formatting of tuple element scopes to 1-based "[x/n]".

### Fixed
- Fixed mismatch in tuple scopes between serialiser and derserialiser.
- Fixed incorrect compacting of seq and map scopes.
- Fixed missing scope around bytes.

## Serde Spaniel 0.2.0 (2021-01-20)

### Added
- Added serialiser and to\_prompt utility functions.

### Changed
- Removed Read/WriteLine traits.
- Updated RustyLine dependency to 7.1.

### Fixed
- Fixed bad prompt in bytes deserialiser.
