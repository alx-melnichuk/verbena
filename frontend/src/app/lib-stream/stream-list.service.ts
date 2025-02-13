import { Injectable } from '@angular/core';
import { HttpErrorResponse } from '@angular/common/http';

import { PageInfo, PageInfoUtil } from '../common/page-info';
import { AlertService } from '../lib-dialog/alert.service';
import { SearchStreamDto, StreamDto, StreamListDto } from './stream-api.interface';
import { StreamService } from './stream.service';
import { HttpErrorUtil } from '../utils/http-error.util';

const CN_DEFAULT_LIMIT = 7; // 10;
const CN_LIMIT_RECEIVING_PAGES = 59;

@Injectable({
    providedIn: 'root'
})
export class StreamListService {
    // "Future Streams"
    public futureStreamLoading = false;
    public futureStreamsDto: StreamDto[] = [];
    public futurePageInfo: PageInfo = this.createFuturePageInfo();

    // "Past Streams"
    public pastStreamLoading = false;
    public pastStreamsDto: StreamDto[] = [];
    public pastPageInfo: PageInfo = this.createPastPageInfo();

    private datetimeLastUpdate: Date | null = null;

    constructor(
        private alertService: AlertService,
        private streamService: StreamService,
    ) {
    }

    // "Future Streams"

    /** Clear array of "Future Stream". */
    public clearFutureStream(): void {
        this.futureStreamsDto = [];
        this.futurePageInfo = this.createFuturePageInfo();
    }
    /** Search for the next page of the "Future Stream". */
    public searchNextFutureStream(userId?: number | undefined): Promise<StreamListDto | HttpErrorResponse | undefined> {
        const delta = !this.datetimeLastUpdate ? 0 : Math.abs(this.dateDifferenceInSeconds(new Date(), this.datetimeLastUpdate));
        if (delta > CN_LIMIT_RECEIVING_PAGES) {
            // Need to reset existing data and get new data for 1st page.
            this.clearFutureStream();
            this.clearAndSearchDataNextPastStream(userId);
        }
        return this.searchDataNextFutureStream(userId);
    }

    // "Past Streams"

    /** Clear array of "Past Stream". */
    public clearPastStream(): void {
        this.pastStreamsDto = [];
        this.pastPageInfo = this.createPastPageInfo();
    }
    /** Search for the next page of the "Past Stream". */
    public searchNextPastStream(userId?: number | undefined): Promise<StreamListDto | HttpErrorResponse | undefined> {
        const delta = !this.datetimeLastUpdate ? 0 : Math.abs(this.dateDifferenceInSeconds(new Date(), this.datetimeLastUpdate));
        if (delta > CN_LIMIT_RECEIVING_PAGES) {
            // Need to reset existing data and get new data for 1st page.
            this.clearPastStream();
            this.clearAndSearchDataNextFutureStream(userId);
        }
        return this.searchDataNextPastStream(userId);
    }

    // ** Private API **

    private createFuturePageInfo(): PageInfo {
        return PageInfoUtil.create({ page: 0, limit: CN_DEFAULT_LIMIT });
    }
    private createPastPageInfo(): PageInfo {
        return PageInfoUtil.create({ page: 0, limit: CN_DEFAULT_LIMIT, orderDirection: 'desc' });
    }
    private dateDifferenceInSeconds(datetimeA: Date, datetimeB: Date): number {
        // This will give difference in milliseconds
        const difference = datetimeA.getTime() - datetimeB.getTime();
        return Math.round(difference / 1000);
    }
    private async getNextPageStreams(pageInfo: PageInfo, isFuture: boolean, titleErr?: string, userId?: number | undefined
    ): Promise<StreamListDto | HttpErrorResponse | undefined> {
        const pages = pageInfo.pages;
        const nextPage = pageInfo.page + 1;
        const isNextPage = ((pages === -1) || (pageInfo.page !== nextPage && nextPage <= pages));
        if (!isNextPage) {
            return Promise.resolve(undefined);
        }
        const oldOrderDir = pageInfo.orderDirection;
        const orderDirection: 'asc' | 'desc' | undefined = (oldOrderDir == 'asc' ? 'asc' : (oldOrderDir == 'desc' ? 'desc' : undefined));

        let searchStream: SearchStreamDto = {
            userId,
            [(isFuture ? 'futureStarttime' : 'pastStarttime')]: (new Date()).toISOString(),
            orderDirection,
            page: pageInfo.page + 1,
            limit: pageInfo.limit
        };

        try {
            const result = await this.streamService.getStreams(searchStream);
            return result;
        } catch (error: unknown) {
            this.alertService.showError(HttpErrorUtil.getMsgs(error as HttpErrorResponse)[0], titleErr);
            throw error;
        }
    }
    // "Future Streams"
    private async clearAndSearchDataNextFutureStream(userId?: number | undefined) {
        this.clearFutureStream();
        await this.searchDataNextFutureStream(userId);
    }
    /* Execute a query to retrieve data from the next page of the "Future Stream". */
    private async searchDataNextFutureStream(userId?: number | undefined): Promise<StreamListDto | HttpErrorResponse | undefined> {
        let result: StreamListDto | HttpErrorResponse | undefined;
        this.futureStreamLoading = true;
        try {
            result = await this.getNextPageStreams(this.futurePageInfo, true, 'stream_list.error_get_future_streams', userId);
            const futureStreamListDto = (result as StreamListDto);
            this.futurePageInfo = PageInfoUtil.create(futureStreamListDto);
            if (this.futureStreamsDto.length == 0) {
                this.datetimeLastUpdate = new Date();
            }
            this.futureStreamsDto = this.futureStreamsDto.concat(futureStreamListDto.list);
        } finally {
            this.futureStreamLoading = false;
        }
        return result;
    }
    // "Past Streams"
    private async clearAndSearchDataNextPastStream(userId?: number | undefined) {
        this.clearPastStream();
        await this.searchDataNextPastStream(userId);
    }
    /* Execute a query to retrieve data from the next page of the "Past Stream". */
    private async searchDataNextPastStream(userId?: number | undefined): Promise<StreamListDto | HttpErrorResponse | undefined> {
        let result: StreamListDto | HttpErrorResponse | undefined;
        this.pastStreamLoading = true;
        try {
            result = await this.getNextPageStreams(this.pastPageInfo, false, 'stream_list.error_get_past_streams', userId);
            const pastStreamListDto = (result as StreamListDto);
            this.pastPageInfo = PageInfoUtil.create(pastStreamListDto);
            if (this.pastStreamsDto.length == 0) {
                this.datetimeLastUpdate = new Date();
            }
            this.pastStreamsDto = this.pastStreamsDto.concat(pastStreamListDto.list);
        } finally {
            this.pastStreamLoading = false;
        }
        return result;
    }
}
