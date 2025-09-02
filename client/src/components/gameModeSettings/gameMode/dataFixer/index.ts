import { CurrentFormat, GameModeStorage, ShareableGameMode } from "..";
import { Settings } from "../../../../game/localStorage";
import { Failure, ParseResult, Success, isFailure } from "../parse"
import initial from "./initial"
import v0 from "./v0"
import v1 from "./v1"
import v2 from "./v2"
import v3 from "./v3"
import v4 from "./v4"
import v5 from "./v5"
import v6 from "./v6"

/// A version converter from a specified version to the next version
export type VersionConverter = {
    matchSettings?: (json: NonNullable<any>) => boolean,
    convertSettings?: (json: NonNullable<any>) => ParseResult<NonNullable<any>>,

    matchGameModeStorage?: (json: NonNullable<any>) => boolean,
    convertGameModeStorage: (json: NonNullable<any>) => ParseResult<NonNullable<any>>,

    matchShareableGameMode?: (json: NonNullable<any>) => boolean,
    convertShareableGameMode: (json: NonNullable<any>) => ParseResult<NonNullable<any>>,
}

type ConverterMap = {
    "GameModeStorage": GameModeStorage,
    "ShareableGameMode": ShareableGameMode,
    "Settings": Settings,
}

const VERSION_CONVERTERS: Record<string, VersionConverter> = { initial, v0, v1, v2, v3, v4, v5, v6 }

/// This converter is the latest version, and acts as a verification step.
export const LATEST_VERSION_STRING: CurrentFormat = "v6";
const LATEST_VERSION: VersionConverter = VERSION_CONVERTERS[LATEST_VERSION_STRING];

export default function parseFromJson<T extends keyof ConverterMap>(type: T, json: NonNullable<any>): ParseResult<ConverterMap[T]> {
    function isCorrectConverter(json: NonNullable<any>, converter: VersionConverter, version: string) {
        return (typeof json === "object" && !Array.isArray(json) && json.format === version) || 
            (converter[`match${type}`] !== undefined && converter[`match${type}`]!(json))
    }

    function convert(converter: VersionConverter, json: NonNullable<any>) {
        const convertFunction = converter[`convert${type}`] ?? (j => Success(j))
        return convertFunction(json);
    }

    const MAX_ITERATIONS = 1000;
    let currentJson = json;
    for(let i = 0; i < MAX_ITERATIONS; i++) {
        const converterEntry = Object.entries(VERSION_CONVERTERS)
            // eslint-disable-next-line no-loop-func
            .find(([version, converter]) => isCorrectConverter(currentJson, converter, version));
    
        if (converterEntry === undefined) {
            return Failure("unsupportedFormat", currentJson);
        }

        const converter = converterEntry[1];

        const result = convert(converter, currentJson);
        if (isFailure(result))
            return result;

        currentJson = result.value;

        if (converter === LATEST_VERSION) {
            return Success(currentJson);
        }
    }
    return Failure("failedToParse", json);
}