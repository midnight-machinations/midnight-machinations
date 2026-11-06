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

export function isFailure(result: ParseResult<any>): result is ParseFailure {
    return result.type === "failure";
}

export function Success<T>(result: T): ParseSuccess<T> {
    return {
        type: "success",
        value: result,
    };
}

export function Failure(reason: ParseFailure["reason"], snippet: any): ParseFailure {
    return {
        type: "failure",
        reason,
        snippet: JSON.stringify(snippet),
        toString: () => `${reason}: ${snippet}`,
    };
}

export function parseJsonObject(jsonString: string): NonNullable<any> | null {
    let json: any;
    try {
        json = JSON.parse(jsonString);
    } catch {
        return null;
    }

    return json;
}
