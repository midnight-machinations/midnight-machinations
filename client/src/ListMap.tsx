export type ListMapData<K, V> = [K, V][]

export default class ListMap<K, V> {
    list: ListMapData<K, V>;
    equalsFn: ((k1: K, k2: K)=>boolean);
    constructor(
        list: ListMapData<K, V> = [],
        equalsFn: ((k1: K, k2: K)=>boolean) = (k1, k2)=>k1 === k2
    ) {
        this.list = list;
        this.equalsFn = equalsFn;
    }
    fixUnique(): ListMap<K, V> {
        const list = this.list;
        this.list = [];
        for(const [key, val] of list){
            this.insert(key, val)
        }
        return this;
    }
    contains(key: K): boolean {
        for (const kvp of this.list) {
            if (this.equalsFn(kvp[0], key)) {
                return true
            }
        }
        return false
    }
    get(key: K): V | null {
        for (const [k, v] of this.list) {
            if (this.equalsFn(k, key)) {
                return v
            }
        }
        return null
    }
    insert(key: K, value: V) {
        this.list = this.list.filter(([k, v]) => !this.equalsFn(k, key));
        this.list.push([key, value])
    }
    delete(key: K) {
        this.list = this.list.filter(([k, v]) => !this.equalsFn(k, key))
    }
    entries(): ListMapData<K, V> {
        return this.list
    }
    keys(): K[] {
        return this.list.map(([k, v]) => k)
    }
    values(): V[] {
        return this.list.map(([k, v]) => v)
    }


}