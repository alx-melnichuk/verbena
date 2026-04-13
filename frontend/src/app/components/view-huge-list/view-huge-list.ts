import { CommonModule, NgTemplateOutlet } from "@angular/common";
import {
    AfterViewInit, ChangeDetectionStrategy, Component, ContentChild, ElementRef, EventEmitter, inject, Input, OnChanges,
    OnDestroy, Output, SimpleChanges, TemplateRef, ViewChild, ViewEncapsulation
} from "@angular/core";

export interface ItemView {
    id: number;
}

export interface ItemViewPage {
    list: ItemView[];
    limit: number;
    page: number;
    pages: number;
}

export const DEBOUNCE = 500; // 200;
export const THRESHOLD_UP_DEFAULT = 0.002;
export const THRESHOLD_DOWN_DEFAULT = 0.998;

type DirectionType = "column" | "column-reverse" | "row" | "row-reverse";

let uniqueIdCounter = 0;

@Component({
    selector: "app-view-huge-list",
    exportAs: "appViewHugeList",
    standalone: true,
    imports: [CommonModule, NgTemplateOutlet],
    templateUrl: "./view-huge-list.html",
    styleUrl: "./view-huge-list.scss",
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class ViewHugeList implements OnChanges, AfterViewInit, OnDestroy {
    private elementRef: ElementRef<HTMLElement> = inject<ElementRef<HTMLElement>>(ElementRef);

    @Input()
    public id = `vhl-id${uniqueIdCounter++}`;
    @Input()
    public debounce: number | null | undefined;
    @Input()
    public deletedId: number | null | undefined;
    @Input()
    public isReverse: boolean | null | undefined; // Only for direction: "column", "row".
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
    private direction: DirectionType = "column";
    private limit: number = 0;
    private pageEnd: number = 0;
    private get pageBgn(): number {
        const delta = this.dataList.length > 0 && this.limit > 0 ? Math.ceil(this.dataList.length / this.limit) - 1 : 0;
        return this.pageEnd > delta ? this.pageEnd - delta : this.pageEnd;
    }
    private set pageBgn(value: number) { }
    private pages: number = 0;
    private rateCurrX: number = -1;
    private rateCurrY: number = -1;
    private ratePrevX: number = 0;
    private ratePrevY: number = 0;
    private sizeRows: number = 0; // The final maximum size of rows in the buffer (a multiple of "limit").

    // Set passive to true to prevent the browser from waiting for this event handler to complete.
    private scrollHandlerFn = (event: Event) => this.scrollHandler(event);
    private scrollIsLoked: boolean = false;
    private scrollOptions: AddEventListenerOptions = { passive: true };

    constructor() {
        const elem = this.elementRef.nativeElement;
        let direction: DirectionType | null = null;
        direction = elem.getAttribute("vhl-col") != null ? "column" : direction;
        direction = elem.getAttribute("vhl-col-rev") != null ? "column-reverse" : direction;
        direction = elem.getAttribute("vhl-row") != null ? "row" : direction;
        direction = elem.getAttribute("vhl-row-rev") != null ? "row-reverse" : direction;
        if (direction == null) {
            elem.setAttribute("vhl-col", "");
            direction = "column";
        }
        this.direction = direction;
    }

    ngOnChanges(changes: SimpleChanges): void {
        let isScrollHandling: boolean = false;
        let isSetScrollMin: boolean | null = null;

        if (!!changes["deletedId"] && !!this.deletedId) {
            const index = this.getItemById(this.dataList, this.deletedId);
            if (index > -1) {
                const len = this.dataList.length;
                this.dataList.splice(index, 1);
                if (this.pageEnd < this.pages) {
                    const rowIdx = (this.pageBgn - 1) * this.limit + len;
                    this.scrollIsLoked = true; // Disable scroll event handling.
                    Promise.resolve().then(() => this.loadPage.emit({ page: rowIdx, limit: 1 }));
                } else if (this.dataList.length <= (this.sizeRows - this.limit)) {  // pageEnd == pages
                    this.pages--;
                    this.pageEnd = this.pages;
                    if (this.pageBgn > 1) {
                        this.scrollIsLoked = true; // Disable scroll event handling.
                        Promise.resolve().then(() => this.loadPage.emit({ page: this.pageBgn - 1, limit: this.limit }));
                    }
                }
            }
        }
        if (this.limit == 0 && !!changes["itemPage"] && !!this.itemPage && this.itemPage.limit > 1) {
            this.limit = this.itemPage.limit;
        }
        if (this.sizeRows == 0 && !!changes["itemPage"] && !!this.itemPage && this.itemPage.limit > 1) {
            this.sizeRows = Math.floor(this.maxSizeRows / this.itemPage.limit) * this.itemPage.limit;
        }
        if (!!changes["itemPage"] && !!this.itemPage && this.itemPage.limit > 1) {
            const isDirColOrRow = ["column", "row"].indexOf(this.direction) > -1;
            if (!!this.isReverse && isDirColOrRow) {
                this.itemPage.list.reverse();
            }
            if (this.isReset) {
                this.clear();
                isSetScrollMin = !this.isReverse ? true : false;
            }
            if (!this.scrollIsLoked) {
                this.scrollIsLoked = true;
            }

            this.dataList = this.prepareItems(
                !!this.isReverse && isDirColOrRow, this.dataList, this.itemPage, this.pageBgn, this.pageEnd, this.sizeRows, this.limit);

            if (this.itemPage.page == this.pageBgn - 1) {
                this.pageEnd = this.pageEnd - 1;
            } else if (this.pageEnd + 1 == this.itemPage.page) {
                this.pageEnd = this.itemPage.page;
            }
            this.pages = this.itemPage.pages;

            // Processing scroll position after adding data.
            isScrollHandling = true;
        }
        if (!!changes["itemPage"] && !!this.itemPage && this.itemPage.limit == 1) {
            if (!this.scrollIsLoked) {
                this.scrollIsLoked = true;
            }

            if (this.itemPage.list.length > 0) {
                const el: HTMLElement | undefined = this.container?.nativeElement;
                if (!!this.isReverse && ["column", "row"].indexOf(this.direction) > -1) {
                    this.dataList = [this.itemPage.list[0]].concat(this.dataList);
                } else {
                    this.dataList.push(this.itemPage.list[0]);
                }
            }
            const countRows = this.itemPage.pages;
            const pages2 = Math.ceil(countRows / this.limit);
            if (pages2 < this.pages) {
                this.pages = pages2;
            }

            // Processing scroll position after adding data.
            isScrollHandling = true;
        }

        // 
        if (isScrollHandling) {
            // Move the slider by 1 pixel so that the first (last) visible element of the data insertion field remains in place.
            const e: HTMLElement | undefined = this.container?.nativeElement;
            const dL = this.direction == "row" ? 1 : (this.direction == "row-reverse" ? -1 : 0);
            const width = (e?.scrollWidth || 0) - (e?.clientWidth || 0);

            if (!!e && dL != 0 && width > 0) {
                if (!this.isReverse && isSetScrollMin != null) {
                    e.scrollLeft = isSetScrollMin ? 0 : dL * width;
                }
                e.scrollLeft = e.scrollLeft == 0 ? e.scrollLeft + dL : (e.scrollLeft == dL * width ? e.scrollLeft - dL : e.scrollLeft);
            }
            const dT = this.direction == "column" ? 1 : (this.direction == "column-reverse" ? -1 : 0);
            const height = (e?.scrollHeight || 0) - (e?.clientHeight || 0);
            if (!!e && dT != 0 && height > 0) {
                if (!this.isReverse && isSetScrollMin != null) {
                    e.scrollTop = isSetScrollMin ? 0 : dT * height;
                }
                e.scrollTop = e.scrollTop == 0 ? e.scrollTop + dT : (e.scrollTop == dT * height ? e.scrollTop - dT : e.scrollTop);
            }

            // Trigger after the next repaint.
            requestAnimationFrame(() => {
                // Move the slider by 1 pixel so that the first (last) visible element of the data insertion field remains in place.
                const width = (e?.scrollWidth || 0) - (e?.clientWidth || 0);
                if (!!e && dL != 0 && width > 0) {
                    if (!!this.isReverse && isSetScrollMin != null) {
                        e.scrollLeft = isSetScrollMin ? 0 : dL * width;
                    }
                    e.scrollLeft = e.scrollLeft == 0 ? e.scrollLeft + dL : (e.scrollLeft == dL * width ? e.scrollLeft - dL : e.scrollLeft);
                }
                const height = (e?.scrollHeight || 0) - (e?.clientHeight || 0);
                if (!!e && dT != 0 && height > 0) {
                    if (!!this.isReverse && isSetScrollMin != null) {
                        e.scrollTop = isSetScrollMin ? 0 : dT * height;
                    }
                    e.scrollTop = e.scrollTop == 0 ? e.scrollTop + dT : (e.scrollTop == dT * height ? e.scrollTop - dT : e.scrollTop);
                }

                // When the new data in the "dataList" array is already displayed,
                // we enable scroll event handling.
                this.scrollIsLoked = false;
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
        if (!event.target) {
            return;
        }
        const elem: HTMLElement = event.target as HTMLElement;
        // Sometimes the maximum value of scrollTop differs from the value (scrollHeight - clientHeight) by 1px.
        // And then there will never be an event of reaching the upper limit.
        // To handle this error, 1 is subtracted from the value (scrollHeight - clientHeight).
        const scrollValueX = elem.scrollWidth - elem.clientWidth - 1;
        const scrollValueY = elem.scrollHeight - elem.clientHeight - 1;

        if (scrollValueX > 0) {
            this.ratePrevX = this.rateCurrX;
            this.rateCurrX = Math.round((Math.abs(elem.scrollLeft) / scrollValueX) * 1000) / 1000;
        } else if (scrollValueY > 0) {
            this.ratePrevY = this.rateCurrY;
            this.rateCurrY = Math.round((Math.abs(elem.scrollTop) / scrollValueY) * 1000) / 1000;
        }

        if (!!this.scrollIsLoked || !!this.isScrollLock) {
            return;
        }
        const rateStart = this.thresholdUp || THRESHOLD_UP_DEFAULT;
        const rateFinish = this.thresholdDown || THRESHOLD_DOWN_DEFAULT;
        let isRateUp = false;
        let isRateDw = false;
        if (scrollValueX > 0) {
            isRateUp = this.ratePrevX > this.rateCurrX && this.ratePrevX > rateStart && rateStart >= this.rateCurrX;
            isRateDw = this.ratePrevX < this.rateCurrX && this.ratePrevX < rateFinish && rateFinish <= this.rateCurrX;
        } else if (scrollValueY > 0) {
            isRateUp = this.ratePrevY > this.rateCurrY && this.ratePrevY > rateStart && rateStart >= this.rateCurrY;
            isRateDw = this.ratePrevY < this.rateCurrY && this.ratePrevY < rateFinish && rateFinish <= this.rateCurrY;
        }
        if (!isRateUp && !isRateDw) {
            return;
        }
        if (!!this.isReverse && (["column", "row"].indexOf(this.direction) > -1)) {
            const isRateUpVal = isRateUp;
            isRateUp = isRateDw;
            isRateDw = isRateUpVal;
        }
        let newPage: number = 0;
        if (isRateUp && this.pages > 0 && this.pageBgn > 1) {
            newPage = this.pageBgn - 1;
        } else if (isRateDw && 0 <= this.pageEnd && (this.pages == 0 || this.pageEnd < this.pages)) {
            newPage = this.pageEnd + 1;
        }
        if (newPage > 0) {
            this.scrollIsLoked = true; // Disable scroll event handling.
            Promise.resolve().then(() => this.loadPage.emit({ page: newPage, limit: this.limit }));
        }
    }

    private prepareItems(
        isRvr: boolean, list: ItemView[], itemPage: ItemViewPage, pgBgn: number, pgEnd: number, sizeRows: number, limit: number
    ) {
        let result: ItemView[] = [];
        let currList = list.concat([]);
        const isItemPageEqPgBgn = itemPage.page == pgBgn - 1;
        const isItemPageEqPgEnd = pgEnd + 1 == itemPage.page;

        if (sizeRows > 0 && list.length + itemPage.list.length > sizeRows) {
            if ((!isRvr && isItemPageEqPgBgn) || (isRvr && isItemPageEqPgEnd)) {
                currList = currList.slice(0, sizeRows - limit); // Delete "end" page lines from the list.
            } else if ((!isRvr && isItemPageEqPgEnd) || (isRvr && isItemPageEqPgBgn)) {
                currList = currList.slice(list.length - (sizeRows - limit), undefined); // Delete "begin" page lines from the list.
            }
        }

        if ((!isRvr && isItemPageEqPgBgn) || (isRvr && isItemPageEqPgEnd)) {
            result = itemPage.list.concat(currList); // Add the previous page to the top of the list ("begin" page).
        } else if ((!isRvr && isItemPageEqPgEnd) || (isRvr && isItemPageEqPgBgn)) {
            result = currList.concat(itemPage.list); // Add the next page to the end of the list ("end" page).
        } else if (pgBgn <= itemPage.page && itemPage.page <= pgEnd) {
            const idx1 = limit * ((!isRvr ? itemPage.page - pgBgn : pgEnd - itemPage.page));
            const chunk1 = currList.slice(0, idx1);
            const chunk2 = currList.slice(idx1 + limit, undefined);
            result = chunk1.concat(itemPage.list, chunk2);
        }
        return result;
    }
    private getItemById(list: ItemView[], deletedId: number): number {
        let index = -1;
        for (let n = 0; n < list.length && index == -1; n++) {
            if (deletedId == list[n].id) {
                index = n;
            }
        }
        return index;
    }
}
