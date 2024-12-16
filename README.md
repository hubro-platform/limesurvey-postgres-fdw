# LimeSurvey Postgres Wrapper

This project implements Wasm Foreign Data Wrapper (FDW) for a LimeSurvey data source. This allows for seamless
integration, querying, and manipulation of survey data stored in LimeSurvey format from within a Postgres database.

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

A [Wasm Interface Type](https://github.com/bytecodealliance/wit-bindgen) (WIT) defines the interfaces between the Wasm
FDW (guest) and the Wasm runtime (host). For example, the `http.wit` defines the HTTP related types and functions can be
used in the guest, and the `routines.wit` defines the functions the guest needs to implement.

## Installation

Enable Wrappers extension

```postgresql
create extension if not exists wrappers with schema extensions;

create foreign data wrapper wasm_wrapper
  handler wasm_fdw_handler
  validator wasm_fdw_validator;
```

Install LimeSurvey wrapper from Github

```postgresql
create server limesurvey
    foreign data wrapper wasm_wrapper
    options (
        fdw_package_url 'https://github.com/hubro-platform/limesurvey-fdw/releases/download/v0.1.0/limesurvey_fdw.wasm',
        fdw_package_name 'hubroplatform:limesurvey-fdw',
        fdw_package_version '0.1.0',
        fdw_package_checksum 'abc1234567abcdef1234567abc1234567abcdef1234567abcdef1234567abcdef',
        api_url 'https://example.com/admin/remotecontrol',
        api_key 'your_limesurvey_api_key'
        );

create schema limesurvey;

create foreign table limesurvey.surveys (
    id text,
    title text,
    start_date timestamptz,
    end_date timestamptz,
    owner text,
    responses jsonb,
    attrs jsonb
    )
    server limesurvey
    options (
        object 'Survey',
        rowid_column 'id'
        );
```

## License

[Apache License Version 2.0](./LICENSE)
