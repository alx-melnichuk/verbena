import { Injectable } from '@angular/core';
import { SocketApiService } from './socket-api.service';

@Injectable({
    providedIn: 'root'
})
export class SocketService {

    constructor(private socketApiService: SocketApiService) {
        console.log(`SocketService()`); // #
    }

    public connect(): void {
        this.socketApiService.connect('ws');
    }

    public disconnect(): void {
        this.socketApiService.disconnect();
    }
}
