/**
 * JSON read from storage, links, or older formats, before migrations check its shape.
 * Migrations read fields that only older formats have, so this stays untyped on purpose.
 */
// biome-ignore lint/suspicious/noExplicitAny: see above
export type UntypedJson = any;

export type ParseSuccess<T> = {
    type: "success";
    value: T;
};

export type ParseFailure = {
    type: "failure";
    reason: string;
    snippet: string;
    toString: () => string;
};
export type ParseResult<T> = ParseSuccess<T> | ParseFailure;

export function isFailure(result: ParseResult<unknown>): result is ParseFailure {
    return result.type === "failure";
}

export function Success<T>(result: T): ParseSuccess<T> {
    return {
        type: "success",
        value: result,
    };
}

export function Failure(reason: ParseFailure["reason"], snippet: unknown): ParseFailure {
    return {
        type: "failure",
        reason,
        snippet: JSON.stringify(snippet),
        toString: () => `${reason}: ${snippet}`,
    };
}

export function parseJsonObject(jsonString: string): UntypedJson | null {
    let json: UntypedJson;
    try {
        json = JSON.parse(jsonString);
    } catch {
        return null;
    }

    return json;
}
