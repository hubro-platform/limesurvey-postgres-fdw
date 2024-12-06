# FHIR Postgres Wrapper

This project implements Wasm Foreign Data Wrapper (FDW) for a FHIR (Fast Healthcare
Interoperability Resources) data source. This allows for seamless
integration, querying, and manipulation of healthcare data stored in FHIR format from within a Postgres database.

## Project Structure

```bash
├── src
│   └── lib.rs
├── supabase-wrappers-wit
│   ├── http.wit
│   ├── jwt.wit
│   ├── routines.wit
│   ├── stats.wit
│   ├── time.wit
│   ├── types.wit
│   ├── utils.wit
│   └── world.wit
└── wit                  
    └── world.wit
```

A [Wasm Interface Type](https://github.com/bytecodealliance/wit-bindgen) (WIT) defines the interfaces between the Wasm FDW (guest) and the Wasm runtime (host). For example, the `http.wit` defines the HTTP related types and functions can be used in the guest, and the `routines.wit` defines the functions the guest needs to implement.

## Installation

Enable Wrappers extension

```postgresql
create extension if not exists wrappers with schema extensions;

create foreign data wrapper wasm_wrapper
  handler wasm_fdw_handler
  validator wasm_fdw_validator;
```

Install FHIR wrapper from Github

```postgresql
create server fhir
    foreign data wrapper wasm_wrapper
    options (
        fdw_package_url 'https://github.com/hubro-platform/fhir-postgres-fdw/releases/download/v0.2.0/fhir_postgres_fdw.wasm',
        fdw_package_name 'hubroplatform:fhir-postgres-fdw',
        fdw_package_version '0.2.0',
        fdw_package_checksum '338674c4c983aa6dbc2b6e63659076fe86d847ca0da6d57a61372b44e0fe4ac9',
        fhir_url 'https://hapi.fhir.org/baseR4'
        );

create schema fhir;

create foreign table fhir.observations (
    id text,
    effectiveStart timestamptz,
    effectiveEnd timestamptz,
    loincCode text,
    subject text,
    value double precision,
    unit text,
    attrs jsonb
    )
    server fhir
    options (
        object 'Observation',
        rowid_column 'id'
        );
```

## License

[Apache License Version 2.0](./LICENSE)
