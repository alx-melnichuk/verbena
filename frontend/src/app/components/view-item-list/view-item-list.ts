import { CommonModule, NgTemplateOutlet } from "@angular/common";
import {
    ChangeDetectionStrategy, Component, ContentChild, ElementRef, EventEmitter, Input, Output, TemplateRef, ViewChild, ViewEncapsulation
} from "@angular/core";

export interface ItemView {
    id: unknown;
}

export interface LoadParams {
    isAddTop: boolean;
    count: number;
}

export const THRESHOLD_UP_DEFAULT = 0.01;
export const THRESHOLD_DOWN_DEFAULT = 0.998;

const DELAY_DEBOUNCE = 50;
const MODE_CHECK_RATE = "CHECK_RATE";
const MODE_STABILIZE_SCROLL = "STABILIZE_SCROLL";
const MODE_LOAD_ITEM = "LOAD_ITEM";

interface ModeCheckRate {
    type: "CHECK_RATE";
}
interface ModeStabilizeScroll {
    type: "STABILIZE_SCROLL";
    isSetScrollMin: boolean;
}
interface ModeLoadItem {
    type: "LOAD_ITEM";
    isAddTop: boolean;
    count: number;
}

type ModeType = ModeCheckRate | ModeStabilizeScroll | ModeLoadItem;

export function debounceFnc(context: any, func: (...arg0: any) => void, timeout: number): (...args: any) => void {
    let timer: number | undefined;
    return (...args: any) => {
        if (timer != undefined) {
            window.clearTimeout(timer);
        }
        timer = window.setTimeout(() => {
            timer = undefined;
            func.apply(context, args);
        }, timeout);
    };
}

let uniqueIdCounter = 0;

@Component({
    selector: "app-view-item-list",
    exportAs: "appViewItemList",
    standalone: true,
    imports: [CommonModule, NgTemplateOutlet],
    templateUrl: "./view-item-list.html",
    styleUrl: "./view-item-list.scss",
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class ViewItemList {
    @Input()
    public id = `vil-${uniqueIdCounter++}`;
    @Input()
    public containerClass: string | null | undefined;
    @Input()
    public isScrollLock: boolean | null | undefined; // Block scroll processing.
    @Input()
    public maxSizeRows: number | null | undefined; // The maximum number of rows in the buffer.
    @Input()
    public paramObj: { [key: string]: unknown } = {};
    @Input()
    public thresholdUp: number | null | undefined = THRESHOLD_UP_DEFAULT; // Takes values ​​from 0 to 1.
    @Input()
    public thresholdDown: number | null | undefined = THRESHOLD_DOWN_DEFAULT; // Takes values ​​from 0 to 1.

    @Output()
    readonly afterScroll: EventEmitter<{ ratioX: number, ratioY: number }> = new EventEmitter();

    @ViewChild("container")
    public container: ElementRef<HTMLElement> | undefined;
    @ContentChild(TemplateRef, { descendants: true, read: TemplateRef, static: false })
    public templateRef: TemplateRef<unknown> | undefined;

    public dataList: ItemView[] = [];
    private followingModes: ModeType[] = [];
    public loadItemFn = (isAddTop: boolean, item: ItemView, count: number) => { };
    public mode: ModeType = this.createModeCheckRate();
    private ratioX: number = -1; // The coefficient of the scroll value by X.
    private ratioY: number = -1; // The coefficient of the scroll value by Y.
    private unitX: number = 0;
    private unitY: number = 0;

    public scrollDbncFn = (mode: ModeType): void => { };

    constructor() {
        this.scrollDbncFn = debounceFnc(this, this.scrollEvnt, DELAY_DEBOUNCE);
    }

    // ** Public API **

    public clear(): void {
        this.dataList = [];
    }
    public setScrollTop(top: number | undefined | null): void {
        const elem: HTMLElement | undefined = this.container?.nativeElement;
        if (!!elem && top != null) {
            elem.scrollTop = top;
            // Stabilizing the scroll position before and after scrolling items in the list.
            this.stabilizeScroll(false);
        }
    }
    public setScrollBottom(bottom: number | undefined | null): void {
        const elem: HTMLElement | undefined = this.container?.nativeElement;
        if (!!elem && bottom != null) {
            elem.scrollTop = elem.scrollHeight - elem.clientHeight - bottom;
            // Stabilizing the scroll position before and after scrolling items in the list.
            this.stabilizeScroll(false);
        }
    }
    public setDataList(dataList: ItemView[]): void {
        this.dataList = dataList;
    }
    /** Stabilizes the scrolling position after shifting or adding/removing items from the list. */
    public stabilizeScroll(isSetScrollMin: boolean): void {
        // Correction of the scroll position after bounce.
        this.scrollEvntCorrection(this.container?.nativeElement, isSetScrollMin);
        this.scrollDbncFn(this.mode = this.createModeStabilizeScroll(isSetScrollMin));
    }
    public createModeCheckRate(): ModeCheckRate {
        return { type: MODE_CHECK_RATE };
    }
    public createModeStabilizeScroll(isSetScrollMin: boolean): ModeStabilizeScroll {
        return { type: MODE_STABILIZE_SCROLL, isSetScrollMin };
    }
    public createModeLoadItem(isAddTop: boolean, count: number): ModeLoadItem {
        return { type: MODE_LOAD_ITEM, isAddTop, count };
    }
    public addNextMode(value: ModeType): void {
        this.followingModes.push(value);
    }
    /** Scroll processing function after rattling (debounce). */
    public scrollEvnt(mode: ModeType): void {
        if (!!this.isScrollLock) {
            return;
        }
        const elem: HTMLElement | undefined = this.container?.nativeElement;

        if (mode.type == MODE_CHECK_RATE) {
            const { isRateUp, isRateDw } = this.scrollEvntCheckRate(elem);
            if (isRateUp || isRateDw) {
                this.followingModes.unshift(this.createModeLoadItem(isRateUp, 0));
            }
        } else if (mode.type == MODE_STABILIZE_SCROLL) {
            // Correction of the scroll position after bounce.
            const isChange = this.scrollEvntCorrection(elem, mode.isSetScrollMin);
            if (isChange) {
                this.followingModes.unshift(this.mode);
            }
        } else if (mode.type == MODE_LOAD_ITEM) {
            this.scrollEvntLoadItem(mode.isAddTop, mode.count);
        }

        const nextMode: ModeType | undefined = this.followingModes.shift();
        if (nextMode != undefined) {
            if (nextMode.type == MODE_STABILIZE_SCROLL) {
                this.scrollDbncFn(this.mode = nextMode);
            } else {
                Promise.resolve().then(() => this.scrollEvnt(this.mode = nextMode));
            }
        } else {
            this.mode = this.createModeCheckRate();
        }
    }

    // ** Private API **

    /** Check the value of the scroll position and request the data if the conditions are met. */
    private scrollEvntCheckRate(elem: HTMLElement | undefined): { isRateUp: boolean, isRateDw: boolean } {
        const { isRateUp, isRateDw, ratioX, ratioY } = this.getScrollRate(elem, this.ratioX, this.ratioY
            , this.thresholdUp || THRESHOLD_UP_DEFAULT, this.thresholdDown || THRESHOLD_DOWN_DEFAULT);
        this.ratioX = ratioX;
        this.ratioY = ratioY;
        return { isRateUp, isRateDw };
    }
    /** Correction of the scroll position after bounce. */
    private scrollEvntCorrection(elem: HTMLElement | undefined, isSetScrollMin: boolean): boolean {
        if (this.unitX == 0 && this.unitY == 0) {
            const { unitX, unitY } = this.getUnitXAndY(elem);
            this.unitX = unitX;
            this.unitY = unitY;
        }
        // Correcting scroll position before or after adding data.
        const isChange = this.scrollPositionCorrection(elem, this.unitX, this.unitY, isSetScrollMin);
        return isChange;
    }
    /** After scrolling is complete, load the required data. */
    private scrollEvntLoadItem(isAddTop: boolean, count: number): void {
        const item = { ...this.dataList[isAddTop ? 0 : this.dataList.length - 1] };
        this.loadItemFn(isAddTop, item, count);
    }
    /** Determine the coefficients of the scroll direction. */
    private getUnitXAndY(e: HTMLElement | undefined): { unitX: number, unitY: number } {
        let unitX = 0;
        let unitY = 0;
        const width = (e?.scrollWidth || 0) - (e?.clientWidth || 0);
        const height = (e?.scrollHeight || 0) - (e?.clientHeight || 0);
        const children = !!e ? Array.from(e.children).filter((n) => n.nodeType === 1) : [];
        if (children.length > 1) {
            const rectFirst = children[0].getBoundingClientRect();
            const rectLast = children[children.length - 1].getBoundingClientRect();
            unitX = width > 0 ? (rectFirst.left < rectLast.left ? 1 : (rectFirst.left > rectLast.left ? -1 : 0)) : 0;
            unitY = height > 0 ? (rectFirst.top < rectLast.top ? 1 : (rectFirst.top > rectLast.top ? -1 : 0)) : 0;
        }
        return { unitX, unitY };
    }
    /** Get scroll coefficients by direction. */
    private getScrollRate(
        elem: HTMLElement | undefined, ratioX: number, ratioY: number, thresholdUp: number, thresholdDown: number
    ): { isRateUp: boolean, isRateDw: boolean, ratioX: number, ratioY: number } {
        // Sometimes the maximum value of scrollTop differs from the value (scrollHeight - clientHeight) by 1px.
        // And then there will never be an event of reaching the upper limit.
        // To handle this error, 1 is subtracted from the value (scrollHeight - clientHeight).
        const scrollValueX = (elem?.scrollWidth || 0) - (elem?.clientWidth || 0) - 1;
        const scrollValueY = (elem?.scrollHeight || 0) - (elem?.clientHeight || 0) - 1;

        const ratePrevX = ratioX;
        const ratePrevY = ratioY;
        let isRateUp: boolean = false;
        let isRateDw: boolean = false;
        if (!!elem && scrollValueX > 0) {
            ratioX = Math.round((Math.abs(elem.scrollLeft) / scrollValueX) * 1000) / 1000;
            isRateUp = ratePrevX > ratioX && ratePrevX > thresholdUp && thresholdUp >= ratioX;
            isRateDw = ratePrevX < ratioX && ratePrevX < thresholdDown && thresholdDown <= ratioX;
        } else if (!!elem && scrollValueY > 0) {
            ratioY = Math.round((Math.abs(elem.scrollTop) / scrollValueY) * 1000) / 1000;
            isRateUp = ratePrevY > ratioY && ratePrevY > thresholdUp && thresholdUp >= ratioY;
            isRateDw = ratePrevY < ratioY && ratePrevY < thresholdDown && thresholdDown <= ratioY;
        }
        return { isRateUp, isRateDw, ratioX, ratioY };
    }
    /** Correcting scroll position before or after adding data.
     * Move the slider by 1 pixel so that the first (last) visible element of 
     * the data insertion field remains in place. */
    private scrollPositionCorrection(elem: HTMLElement | undefined, unitX: number, unitY: number, isSetScrollMin: boolean): boolean {
        let isChange: boolean = false;
        if (!!elem) {
            const oldLeft = elem.scrollLeft;
            const width = elem.scrollWidth - elem.clientWidth;
            // Move the slider by 1 pixel so that the first (last) visible element of the data insertion field remains in place.
            if (width > 0 && unitX != 0) {
                if (isSetScrollMin) {
                    elem.scrollLeft = 0; // Set the scroll value to the minimum position.
                }
                if (elem.scrollLeft == 0) {
                    elem.scrollLeft = elem.scrollLeft + unitX;
                } else if (elem.scrollLeft == unitX * width) {
                    elem.scrollLeft = elem.scrollLeft - unitX;
                }
            }
            const oldTop = elem.scrollTop;
            const height = elem.scrollHeight - elem.clientHeight;
            if (height > 0 && unitY != 0) {
                if (isSetScrollMin) {
                    elem.scrollTop = 0; // Set the scroll value to the minimum position.
                }
                if (elem.scrollTop == 0) {
                    elem.scrollTop = elem.scrollTop + unitY;
                } else if (elem.scrollTop == unitY * height) {
                    elem.scrollTop = elem.scrollTop - unitY;
                }
            }
            isChange = oldLeft != elem.scrollLeft || oldTop != elem.scrollTop;
        }
        return isChange;
    }
}
