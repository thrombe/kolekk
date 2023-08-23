import Innertube, { UniversalCache } from "youtubei.js/web";
import { logg } from "../commands";
import {
    Body,
    fetch as tauri_fetch,
    ResponseType,
    type FetchOptions,
    type HttpVerb
} from '@tauri-apps/api/http';


export async function new_innertube_instance_netlify() {
    let yt = await Innertube.create({
        cache: new UniversalCache(false),
        generate_session_locally: true,
        fetch: async (input, init) => {
            let url: string;
            if (typeof input === 'string') {
                url = input;
            } else if (input instanceof URL) {
                url = input.toString();
            } else {
                url = input.url;
            }

            let headers = init?.headers
                ? new Headers(init.headers)
                : input instanceof Request
                    ? input.headers
                    : new Headers();
            let headers_copy = JSON.stringify([...headers]);

            headers.delete('user-agent');

            const request = new Request(
                url,
                input instanceof Request ? input : undefined
            );
            let req = {
                url: url,
                headers: headers_copy,
                body: init?.body,
                method: request.method
            };
            let res = await fetch('/.netlify/functions/fetch', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(req)
            });
            return res;
        }
    });
    return yt;
}

export async function new_innertube_instance() {
    let yt = await Innertube.create({
        fetch: async (input: RequestInfo | URL, init?: RequestInit) => {
            let url: string;
            if (typeof input === 'string') {
                url = input;
            } else if (input instanceof URL) {
                url = input.toString();
            } else {
                url = input.url;
            }

            let method: HttpVerb = 'GET';
            if (input instanceof Request) {
                method = input.method as HttpVerb;
            }

            let head: Headers = new Headers();
            if (init?.headers) {
                new Headers(init.headers).forEach((v, k) => {
                    // console.log(k, v);
                    head.append(k, v);
                });
                // head = new Headers(init.headers);
            }
            if (input instanceof Request) {
                input.headers.forEach((v, k) => {
                    // console.log(k, v);
                    head.append(k, v);
                });
                // head = input.headers;
            }

            let body: Body | undefined;
            if (init?.body) {
                body = Body.text(init.body.toString());

                // IDK: there is a body thing in 'input' too. input.body
                // if (input instanceof Request && input.body) {
                //     console.log(await input.json())
                // }
            }

            let tauri_fetch_options: FetchOptions = {
                method: method,
                responseType: ResponseType.Text,
                headers: head,
                body: body
            };

            // head.forEach((v, k) => {
            //     console.log(k, v);
            // });
            // console.log(input, init);
            // console.log(url, tauri_fetch_options);

            // - [http | Tauri](https://tauri.app/v1/api/js/http/#fetch)
            let resp = await tauri_fetch(url, tauri_fetch_options);

            // console.log(resp);
            await logg(resp.data as string);
            if (typeof resp.data !== 'string') {
                console.error('AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA');
            }

            let resp_options = {
                status: resp.status,
                headers: new Headers(resp.headers)
            };
            let r = new Response(resp.data as string, resp_options);
            return r;
        }
    });
    return yt;
}
