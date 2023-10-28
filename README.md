# An Audio Silence Compressor for ATC Recordings

## Command Line Usage

`atc [OPTIONS] INPUT [OUTPUT]`

Process the audio from `INPUT` and output to `OUTPUT`. If `OUTPUT` is not specified, overwrite `INPUT`.

### General Options

```text
--help           Print this message and exit
--config FILE    Change configuration file from the default config.toml ("-" for stdin)
--ignore-config  Don't read the default configuration file config.toml
--delay SECONDS  Cut silences that are at least this many seconds long (default 10)
--tts            Use TTS to announce the length of each cut silence (default)
--no-tts         Do not use TTS
```

### Exit Codes

Code | Description
---- | -----------
0    | Success
1    | Operation Error
2    | Command Line Arguments Error
