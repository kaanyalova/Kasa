import { invoke } from "@tauri-apps/api/core";

export class FileServer {
    destroyed: boolean;
    pointer: number;

    private constructor() {
        this.destroyed = false;
        this.pointer = 0;

    }


    destroy() {
        if (this.destroyed == false) {
            invoke('close_server', {
                ptr: this.pointer
            })
            this.destroyed = true;
        }


    }

    static async create(path: String): Promise<FileServer> {
        const ptr: number = await invoke('serve_media', {
            path: path,
        });


        const _new = new FileServer;
        _new.pointer = ptr;

        return _new;

    }


}

const SERVER_URL = "0.0.0.0:3169"


