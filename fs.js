//Compiled from 'https://github.com/ICMC-IDE/icmc-ide/blob/main/src/scripts/resources/fs.ts' using tsc

var __classPrivateFieldGet = (this && this.__classPrivateFieldGet) || function (receiver, state, kind, f) {
    if (kind === "a" && !f) throw new TypeError("Private accessor was defined without a getter");
    if (typeof state === "function" ? receiver !== state || !f : !state.has(receiver)) throw new TypeError("Cannot read private member from an object whose class did not declare it");
    return kind === "m" ? f : kind === "a" ? f.call(receiver) : f ? f.value : state.get(receiver);
};
var _VirtualFileSystemDirectory_directoriesCache, _VirtualFileSystemDirectory_filesCache;
const fileExtensions = new Set(["json", "txt", "c", "asm", "mif", "toml"]);

class VirtualFileSystemObject {
    constructor(name, parent, handle) {
        this.name = name;
        this.handle = handle;
        this.parent = parent;
    }
    get loaded() {
        return this.handle !== undefined;
    }
    get path() {
        return this.parent ? `${this.parent.path}/${this.name}` : this.name;
    }
}
export class VirtualFileSystemDirectory extends VirtualFileSystemObject {
    constructor() {
        super(...arguments);
        _VirtualFileSystemDirectory_directoriesCache.set(this, []);
        _VirtualFileSystemDirectory_filesCache.set(this, []);
    }
    newChildDirectory(name) {
        let directory = __classPrivateFieldGet(this, _VirtualFileSystemDirectory_directoriesCache, "f").find((directory) => directory.name === name);
        if (!directory) {
            directory = new VirtualFileSystemDirectory(name, this);
            __classPrivateFieldGet(this, _VirtualFileSystemDirectory_directoriesCache, "f").push(directory);
        }
        return directory;
    }
    newChildFile(name) {
        let file = __classPrivateFieldGet(this, _VirtualFileSystemDirectory_filesCache, "f").find((file) => file.name === name);
        if (!file) {
            file = new VirtualFileSystemFile(name, this);
            __classPrivateFieldGet(this, _VirtualFileSystemDirectory_filesCache, "f").push(file);
        }
        return file;
    }
    resolveDirectory(path) {
        if (path.length === 0) {
            return this;
        }
        let part = path.shift();
        while (part === "." || part === "") {
            part = path.shift();
        }
        const nextDirectory = this.newChildDirectory(part);
        if (path.length === 0) {
            return nextDirectory;
        }
        return nextDirectory.resolveDirectory(path);
    }
    resolveFile(path) {
        const directory = this.resolveDirectory(path.slice(0, -1));
        const filename = path.at(-1);
        return directory.newChildFile(filename);
    }
    async create(createParents = false) {
        if (this.loaded) {
            return;
        }
        if (createParents) {
            await this.parent.create(true);
        }
        this.handle = await this.parent.handle.getDirectoryHandle(this.name, {
            create: true,
        });
    }
    async load(loadParents = true) {
        if (this.loaded) {
            return;
        }
        if (loadParents) {
            await this.parent.load(true);
        }
        this.handle = await this.parent.handle.getDirectoryHandle(this.name);
    }
    async getDirectory(path, load = true) {
        const directory = this.resolveDirectory(path.split("/"));
        if (load) {
            await directory.load();
        }
        return directory;
    }
    async getFile(path, load = true) {
        const file = this.resolveFile(path.split("/"));
        if (load) {
            await file.load();
        }
        return file;
    }
    async createDirectory(path, createParents = false) {
        const directory = await this.getDirectory(path, false);
        await directory.create(createParents);
        return directory;
    }
    async createFile(path, createParents = false) {
        const file = await this.getFile(path, false);
        await file.create(createParents);
        return file;
    }
    async delete() {
        await this.parent.handle.removeEntry(this.name, { recursive: true });
        this.handle = undefined;
    }
    async copy(directory, name = this.name) {
        const newDirectory = await directory.createDirectory(name);
        for await (const object of this.list()) {
            await object.copy(newDirectory);
        }
        return newDirectory;
    }
    async move(directory, name = this.name) {
        const newDirectoryHandle = (await this.copy(directory, name)).handle;
        await this.delete();
        this.handle = newDirectoryHandle;
        this.name = name;
        this.parent = directory;
    }
    async rename(name) {
        if (name === this.name) {
            return;
        }
        await this.move(this.parent, name);
    }
    async hasDirectory(path) {
        try {
            await this.getDirectory(path);
            return true;
        }
        catch {
            return false;
        }
    }
    async hasFile(path) {
        try {
            await this.getFile(path);
            return true;
        }
        catch {
            return false;
        }
    }
    async *list() {
        for await (const [name, handle] of this.handle.entries()) {
            if (handle instanceof FileSystemFileHandle) {
                const file = this.newChildFile(name);
                file.handle = handle;
                yield file;
            }
            else {
                const directory = this.newChildDirectory(name);
                directory.handle = handle;
                yield directory;
            }
        }
    }
}
_VirtualFileSystemDirectory_directoriesCache = new WeakMap(), _VirtualFileSystemDirectory_filesCache = new WeakMap();
export class VirtualFileSystemFile extends VirtualFileSystemObject {
    get extension() {
        const parts = this.name.split(".");
        if (parts.length === 1) {
            return "txt";
        }
        const extension = parts.at(-1);
        return fileExtensions.has(extension) ? extension : "txt";
    }
    async create(createParents = false) {
        if (this.loaded) {
            return;
        }
        if (createParents) {
            await this.parent.create(true);
        }
        this.handle = await this.parent.handle.getFileHandle(this.name, {
            create: true,
        });
    }
    async load(loadParents = true) {
        if (this.loaded) {
            return;
        }
        if (loadParents) {
            await this.parent.load(true);
        }
        this.handle = await this.parent.handle.getFileHandle(this.name);
    }
    async getReadable() {
        return (await this.handle.getFile()).stream();
    }
    async getFileHandle() {
        return this.handle.getFile();
    }
    async getArrayBuffer() {
        return await (await this.handle.getFile()).arrayBuffer();
    }
    async read() {
        return await (await this.handle.getFile()).text();
    }
    async getWritable() {
        return await this.handle.createWritable();
    }
    async write(data) {
        const handle = await this.handle.createWritable();
        await handle.write(data);
        await handle.close();
    }
    async delete() {
        await this.parent.handle.removeEntry(this.name);
        this.handle = undefined;
    }
    async copy(directory, name = this.name) {
        const content = await this.read();
        const newFile = await directory.createFile(name);
        await newFile.write(content);
        return newFile;
    }
    async move(directory, name = this.name) {
        const newFileHandle = (await this.copy(directory, name)).handle;
        await this.delete();
        this.handle = newFileHandle;
        this.name = name;
        this.parent = directory;
    }
    async rename(name) {
        if (name === this.name) {
            return;
        }
        await this.move(this.parent, name);
    }
}
const ASSETS_PATH = "assets/";
const ASSETS_LIST = ASSETS_PATH + "/assets.json";
export async function loadAssets(directory, loadUserAssets, overwrite = false) {
    const assets = (await (await fetch(ASSETS_LIST)).json());
    await Promise.all([assets.internal, loadUserAssets ? assets.user : []]
        .flat()
        .map(async (asset) => {
        if (!overwrite && (await directory.hasFile(asset))) {
            return;
        }
        const content = await (await fetch(ASSETS_PATH + asset)).arrayBuffer();
        const file = await directory.createFile(asset, true);
        return await file.write(content);
    }));
}

//Fs wrapper (needed, because the class 'Fs' is expected); wraps all the JS in the upward lines 
export class Fs {
    constructor() {
        //root directory with dummy values (customizable if needed)
        this.root = new VirtualFileSystemDirectory("root", undefined, undefined);
    }
    async read(path) {
        try {
            const file = await this.root.getFile(path);
            return await file.read();
        }
        catch {
            return null;
        }
    }
    async write(path, content) {
        const file = await this.root.createFile(path, true);
        await file.write(content);
    }
    async delete(path) {
        try {
            const file = await this.root.getFile(path);
            await file.delete();
        }
        catch {
            // File may not exist
        }
    }
    async files() {
        const result = [];
        for await (const entry of this.root.list()) {
            result.push(entry.name);
        }
        return result;
    }
}
