import { CommonModule, NgTemplateOutlet } from "@angular/common";
import {
    AfterViewInit, ChangeDetectionStrategy, Component, ContentChild, ElementRef, EventEmitter, Input, OnChanges,
    OnDestroy, Output, SimpleChanges, TemplateRef, ViewChild, ViewEncapsulation
} from "@angular/core";

export interface ItemView {
    id: unknown;
}

export interface ItemViewPage {
    list: ItemView[];
    limit: number;
    page: number;
}

export const THRESHOLD_UP_DEFAULT = 0.01;
export const THRESHOLD_DOWN_DEFAULT = 0.998;

const DELAY_BEFORE_LOAD = 100;
const DELAY_FOR_LOAD = 200;

let uniqueIdCounter = 0;

@Component({
    selector: 'app-view-list-by-pages',
    exportAs: "appViewListByPages",
    standalone: true,
    imports: [CommonModule, NgTemplateOutlet],
    templateUrl: './view-list-by-pages.html',
    styleUrl: './view-list-by-pages.scss',
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class ViewListByPages implements OnChanges, AfterViewInit, OnDestroy {
    @Input()
    public id = `vlbp-id${uniqueIdCounter++}`;
    @Input()
    public containerClass: string | null | undefined;
    @Input()
    public deletedId: unknown | null | undefined;
    @Input()
    public isReset: boolean | null | undefined; // Checked only together with "itemPage".
    @Input()
    public isScrollLock: boolean | null | undefined; // Block scroll processing.
    @Input()
    public itemPage: ItemViewPage | null | undefined;
    @Input()
    public maxSizeRows: number = 0; // The maximum size of rows in the buffer.
    @Input()
    public paramObj: { [key: string]: unknown } = {};
    @Input()
    public thresholdUp: number | null | undefined = THRESHOLD_UP_DEFAULT; // Takes values ​​from 0 to 1.
    @Input()
    public thresholdDown: number | null | undefined = THRESHOLD_DOWN_DEFAULT; // Takes values ​​from 0 to 1.

    @Output()
    readonly loadPage: EventEmitter<{ page: number, limit: number }> = new EventEmitter();

    @ContentChild(TemplateRef, { descendants: true, read: TemplateRef, static: false })
    public templateRef: TemplateRef<unknown> | undefined;
    @ViewChild("container")
    public container: ElementRef<HTMLElement> | undefined;

    public dataList: ItemView[] = [];
    private freezeScrollTop: number = 0;
    private limit: number = 0;
    private newPage: number = 0;
    private pageEnd: number = 0;
    private get pageBgn(): number {
        const delta = this.dataList.length > 0 && this.limit > 0 ? Math.ceil(this.dataList.length / this.limit) - 1 : 0;
        return this.pageEnd > delta ? this.pageEnd - delta : this.pageEnd;
    }
    private set pageBgn(value: number) { }
    private rateCurrX: number = -1;
    private rateCurrY: number = -1;
    private sizeRows: number = 0; // The final maximum size of rows in the buffer (a multiple of "limit").
    // Set passive to true to prevent the browser from waiting for this event handler to complete.
    private scrollHandlerFn = (event: Event) => this.scrollHandler(event);
    private pageLoadLocked: number = 0;
    private scrollOptions: AddEventListenerOptions = { passive: true };
    private scrollTimer: number | undefined;
    private unitX: number = 0;
    private unitY: number = 0;

    ngOnChanges(changes: SimpleChanges): void {
        let isScrollHandling: boolean = false;
        let isSetScrollMin: boolean | null = null;

        if (!!changes["deletedId"] && !!this.deletedId) {
            const index = this.getIndexItemById(this.dataList, this.deletedId);
            if (index > -1) {
                let page: number = 0;
                let limit: number = 0;
                const len = this.dataList.length;
                this.dataList.splice(index, 1);
                if (this.dataList.length <= (this.sizeRows - this.limit)) {  // pageEnd == pages
                    this.pageEnd = this.pageEnd - 1;
                    if (this.pageBgn > 1) {
                        page = this.pageBgn - 1;
                        limit = this.limit;
                    }
                } else {
                    page = (this.pageBgn - 1) * this.limit + len;
                    limit = 1;
                }
                if (page > 0 && limit > 0) {
                    // Block scroll position processing to analyze page load.
                    this.pageLoadLocked = Date.now() + DELAY_FOR_LOAD;
                    Promise.resolve().then(() => this.loadPage.emit({ page, limit }));
                }
            }
        }
        if (this.limit == 0 && !!changes["itemPage"] && !!this.itemPage && this.itemPage.limit > 1) {
            this.limit = this.itemPage.limit;
        }
        if (this.sizeRows == 0 && this.maxSizeRows > 0 && !!changes["itemPage"] && !!this.itemPage && this.itemPage.limit > 1) {
            this.sizeRows = Math.floor(this.maxSizeRows / this.itemPage.limit) * this.itemPage.limit;
        }
        if (!!changes["itemPage"] && !!this.itemPage && this.itemPage.limit > 1) {
            if (this.isReset) {
                this.clear();
                isSetScrollMin = true;
            }
            const { items, newPgEnd } = this.prepareItems(
                this.dataList, this.itemPage, this.pageBgn, this.pageEnd, this.sizeRows, this.limit);
            this.dataList = items;
            this.pageEnd = newPgEnd;
            // Processing scroll position after adding data.
            isScrollHandling = true;
        }
        if (!!changes["itemPage"] && !!this.itemPage && this.itemPage.limit == 1) {
            if (this.itemPage.list.length > 0) {
                this.dataList.push(this.itemPage.list[0]);
            }
            // Processing scroll position after adding data.
            isScrollHandling = true;
        }

        if (isScrollHandling) {
            // Block scroll position processing to analyze page load.
            this.pageLoadLocked = Date.now() + DELAY_FOR_LOAD;
            // Move the slider by 1 pixel so that the first (last) visible element of the data insertion field remains in place.
            const e: HTMLElement | undefined = this.container?.nativeElement;
            const width = (e?.scrollWidth || 0) - (e?.clientWidth || 0);
            const height = (e?.scrollHeight || 0) - (e?.clientHeight || 0);
            const children = this.unitX == 0 && this.unitY == 0 && !!e ? Array.from(e.children).filter((n) => n.nodeType === 1) : [];
            if (children.length > 1) {
                const rectFirst = children[0].getBoundingClientRect();
                const rectLast = children[children.length - 1].getBoundingClientRect();
                this.unitX = width > 0 ? (rectFirst.left < rectLast.left ? 1 : (rectFirst.left > rectLast.left ? -1 : 0)) : 0;
                this.unitY = height > 0 ? (rectFirst.top < rectLast.top ? 1 : (rectFirst.top > rectLast.top ? -1 : 0)) : 0;
            }

            if (!!e && width > 0 && this.unitX != 0 && isSetScrollMin === true) {
                e.scrollLeft = 0;
            }
            if (!!e && height > 0 && this.unitY != 0 && isSetScrollMin === true) {
                e.scrollTop = 0;
            }
            // Correcting scroll position before adding data.
            this.correctionScroll(e, this.unitX, this.unitY);

            // Trigger after the next repaint.
            requestAnimationFrame(() => {
                const width = (e?.scrollWidth || 0) - (e?.clientWidth || 0);
                const height = (e?.scrollHeight || 0) - (e?.clientHeight || 0);
                const children = this.unitX == 0 && this.unitY == 0 && !!e ? Array.from(e.children).filter((n) => n.nodeType === 1) : [];
                if (children.length > 1) {
                    const rectFirst = children[0].getBoundingClientRect();
                    const rectLast = children[children.length - 1].getBoundingClientRect();
                    this.unitX = width > 0 ? (rectFirst.left < rectLast.left ? 1 : (rectFirst.left > rectLast.left ? -1 : 0)) : 0;
                    this.unitY = height > 0 ? (rectFirst.top < rectLast.top ? 1 : (rectFirst.top > rectLast.top ? -1 : 0)) : 0;
                }

                // Correcting scroll position before adding data.
                this.correctionScroll(e, this.unitX, this.unitY);
                // When the new data in the "dataList" array is already displayed,
                // unlock scroll position processing to analyze page load.
                this.pageLoadLocked = 0;
            });
        }
    }

    ngAfterViewInit(): void {
        if (!!this.container) {
            this.container.nativeElement.addEventListener("scroll", this.scrollHandlerFn, this.scrollOptions);
        }
    }

    ngOnDestroy(): void {
        if (!!this.container) {
            this.container.nativeElement.removeEventListener("scroll", this.scrollHandlerFn, this.scrollOptions);
        }
    }

    // ** Public API **

    public clear(): void {
        this.dataList = [];
        this.pageEnd = 0;
    }

    // ** Private API **

    private scrollHandler = (event: Event) => {
        const elem: HTMLElement = event.target as HTMLElement;

        let isStartTimer = false;
        if (!!this.scrollTimer) {
            isStartTimer = true;
            if (this.freezeScrollTop != 0 && elem.scrollTop != this.freezeScrollTop) {
                elem.scrollTop = this.freezeScrollTop;
            }
        } else {
            if (!!this.isScrollLock || !event.target) {
                return;
            }
            if (this.pageLoadLocked > 0) {
                // If the current time is less than the lock time, then exit.
                if (Date.now() < this.pageLoadLocked) {
                    return;
                } else {
                    this.pageLoadLocked = 0;
                }
            }
            const { isRateUp, isRateDw, rateCurrX, rateCurrY } = this.getScrollRate(elem, this.rateCurrX, this.rateCurrY,
                this.thresholdUp || THRESHOLD_UP_DEFAULT, this.thresholdDown || THRESHOLD_DOWN_DEFAULT);
            this.rateCurrX = rateCurrX;
            this.rateCurrY = rateCurrY;

            if (!isRateUp && !isRateDw) {
                return;
            }
            if (isRateUp && this.pageBgn > 1) {
                this.newPage = this.pageBgn - 1;
            } else if (isRateDw) {
                this.newPage = this.pageEnd + 1;
            }

            if (this.newPage > 0) {
                // Block scroll position processing to analyze page load.
                this.pageLoadLocked = Date.now() + DELAY_FOR_LOAD;
                // Correcting scroll position before adding data.
                this.correctionScroll(elem, this.unitX, this.unitY);
                this.freezeScrollTop = elem.scrollTop;
                isStartTimer = true;
            }
        }

        if (isStartTimer) {
            if (!!this.scrollTimer) {
                clearTimeout(this.scrollTimer);
            }
            this.scrollTimer = setTimeout(() => {
                clearTimeout(this.scrollTimer);
                this.scrollTimer = undefined;
                this.freezeScrollTop = 0;
                this.loadPage.emit({ page: this.newPage, limit: this.limit });
                this.newPage = 0;
            }, DELAY_BEFORE_LOAD);
        }
    }

    private getScrollRate(
        elem: HTMLElement, rateCurrX: number, rateCurrY: number, thresholdUp: number, thresholdDown: number
    ): { isRateUp: boolean, isRateDw: boolean, rateCurrX: number, rateCurrY: number } {
        // Sometimes the maximum value of scrollTop differs from the value (scrollHeight - clientHeight) by 1px.
        // And then there will never be an event of reaching the upper limit.
        // To handle this error, 1 is subtracted from the value (scrollHeight - clientHeight).
        const scrollValueX = elem.scrollWidth - elem.clientWidth - 1;
        const scrollValueY = elem.scrollHeight - elem.clientHeight - 1;

        const ratePrevX = rateCurrX;
        const ratePrevY = rateCurrY;
        let isRateUp: boolean = false;
        let isRateDw: boolean = false;
        if (scrollValueX > 0) {
            rateCurrX = Math.round((Math.abs(elem.scrollLeft) / scrollValueX) * 1000) / 1000;
            isRateUp = ratePrevX > rateCurrX && ratePrevX > thresholdUp && thresholdUp >= rateCurrX;
            isRateDw = ratePrevX < rateCurrX && ratePrevX < thresholdDown && thresholdDown <= rateCurrX;
        } else if (scrollValueY > 0) {
            rateCurrY = Math.round((Math.abs(elem.scrollTop) / scrollValueY) * 1000) / 1000;
            isRateUp = ratePrevY > rateCurrY && ratePrevY > thresholdUp && thresholdUp >= rateCurrY;
            isRateDw = ratePrevY < rateCurrY && ratePrevY < thresholdDown && thresholdDown <= rateCurrY;
        }
        return { isRateUp, isRateDw, rateCurrX, rateCurrY };
    }
    /** Correcting scroll position before adding data. */
    private correctionScroll(elem: HTMLElement | undefined, unitX: number, unitY: number) {
        const width = (elem?.scrollWidth || 0) - (elem?.clientWidth || 0);
        const height = (elem?.scrollHeight || 0) - (elem?.clientHeight || 0);
        // Move the slider by 1 pixel so that the first (last) visible element of the data insertion field remains in place.
        if (!!elem && width > 0 && unitX != 0) {
            if (elem.scrollLeft == 0) {
                elem.scrollLeft = elem.scrollLeft + unitX;
            } else if (elem.scrollLeft == unitX * width) {
                elem.scrollLeft = elem.scrollLeft - unitX;
            }
        }
        if (!!elem && height > 0 && unitY != 0) {
            if (elem.scrollTop == 0) {
                elem.scrollTop = elem.scrollTop + unitY;
            } else if (elem.scrollTop == unitY * height) {
                elem.scrollTop = elem.scrollTop - unitY;
            }
        }
    }

    private prepareItems(
        list: ItemView[], itemPage: ItemViewPage, pgBgn: number, pgEnd: number, sizeRows: number, limit: number
    ): { items: ItemView[], newPgEnd: number } {
        let items: ItemView[] = [];
        let currList = list.concat([]);
        const isCutOldData = sizeRows > 0 && list.length + itemPage.list.length > sizeRows;
        let newPgEnd: number = pgEnd;
        if (itemPage.page == pgBgn - 1) {
            if (isCutOldData) {
                currList = currList.slice(0, sizeRows - limit); // Delete "end" page lines from the list.
                newPgEnd = newPgEnd - 1;
            }
            items = itemPage.list.concat(currList); // Add the previous page to the top of the list ("begin" page).
        } else if (pgEnd + 1 == itemPage.page || pgEnd == 0) {
            if (isCutOldData) {
                currList = currList.slice(list.length - (sizeRows - limit), undefined); // Delete "begin" page lines from the list.
            }
            items = currList.concat(itemPage.list); // Add the next page to the end of the list ("end" page).
            newPgEnd = itemPage.page;
        } else if (pgBgn <= itemPage.page && itemPage.page <= pgEnd) {
            const idx1 = limit * itemPage.page - pgBgn;
            const chunk1 = currList.slice(0, idx1);
            const chunk2 = currList.slice(idx1 + limit, undefined);
            items = chunk1.concat(itemPage.list, chunk2);
        }
        return { items, newPgEnd };
    }

    private getIndexItemById(list: ItemView[], deletedId: unknown): number {
        let index = -1;
        for (let n = 0; n < list.length && index == -1; n++) {
            if (deletedId == list[n].id) {
                index = n;
            }
        }
        return index;
    }
}
