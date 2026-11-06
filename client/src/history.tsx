/**
 * A utility for keeping track of how we're polling the history
 */
export class HistoryPoller<T> {
    /**
     * -1 indicates we are not using the history.
     */
    index: number;

    constructor() {
        this.index = -1;
    }

    reset(): HistoryPoller<T> {
        this.index = -1;
        return this;
    }

    poll(history: HistoryQueue<T>): T | undefined {
        this.index++;
        const result = history.poll(this.index);
        if (result === undefined) {
            this.index--;
        }
        return result;
    }

    pollPrevious(history: HistoryQueue<T>): T | undefined {
        this.index--;
        if (this.index < 0) {
            this.index = -1;
            return undefined;
        } else {
            const result = history.poll(this.index);
            // History shrunk for some reason. Should be impossible but might as well account for it.
            if (result === undefined) {
                return this.pollPrevious(history);
            } else {
                return result;
            }
        }
    }
}

/**
 * A queue with a max length
 */
export class HistoryQueue<T> {
    private max_length: number;
    private values: T[];

    constructor(max_length: number) {
        this.max_length = max_length;
        this.values = [];
    }

    length(): number {
        return this.values.length;
    }

    poll(index: number): T | undefined {
        return this.values.at(index);
    }

    pop(): T | undefined {
        if (this.values.length === 0) {
            return undefined;
        } else {
            return this.values.splice(0, 1)[0];
        }
    }

    push(value: T) {
        this.values = [value].concat(this.values);
        while (this.values.length > this.max_length) {
            this.values.pop();
        }
    }
}
