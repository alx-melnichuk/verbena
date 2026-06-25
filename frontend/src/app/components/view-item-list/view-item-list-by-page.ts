import { Directive, EventEmitter, inject, Input, OnChanges, Output, SimpleChanges } from '@angular/core';
import { ItemView, ViewItemList } from './view-item-list';

export interface ItemViewPage {
    list: ItemView[];
    page: number;
}

@Directive({
    selector: "app-view-item-list[itemPage]",
    exportAs: "appViewItemListByPage",
    standalone: true,
})
export class ViewItemListByPage implements OnChanges {
    private viewItemList: ViewItemList = inject(ViewItemList);

    @Input()
    public deleteIds: unknown[] | null | undefined;
    @Input()
    public isReset: boolean | null | undefined; // Checked only together with "itemPage".
    @Input()
    public itemPage: ItemViewPage | null | undefined;
    @Input()
    public maxSizeRows: number | null | undefined; // The maximum number of rows in the buffer.
    @Input()
    public minSizeRows: number | null | undefined; // The minimum number of rows in the buffer.
    @Input()
    public rowsOnPage: number | null | undefined; // The number of rows in the page.

    @Output()
    readonly loadPage: EventEmitter<{ page: number, item: ItemView, count: number }> = new EventEmitter();

    private pageEnd: number = 0;
    private get pageBgn(): number {
        const dataListLen = this.viewItemList.dataList.length;
        const rowsOnPage = this.rowsOnPage ?? 0;
        const delta = dataListLen > 0 && rowsOnPage > 0 ? Math.ceil(dataListLen / rowsOnPage) - 1 : 0;
        return this.pageEnd > delta ? this.pageEnd - delta : this.pageEnd;
    }
    private set pageBgn(value: number) { }
    private requestCount: number = 0;

    constructor() {
        this.viewItemList.loadItemFn = this.loadItemPage;
    }

    ngOnChanges(changes: SimpleChanges): void {
        if (!!changes["deleteIds"] && !!this.deleteIds && this.deleteIds.length > 0) {
            const deleteIds = this.deleteIds;
            const isExist = this.viewItemList.dataList.some((val) => deleteIds.indexOf(val.id) > -1);
            if (isExist) {
                const oldCount = this.viewItemList.dataList.length;
                const list = this.viewItemList.dataList.filter((val) => deleteIds.indexOf(val.id) == -1);
                this.viewItemList.setDataList(list);
                this.requestCount = oldCount - this.viewItemList.dataList.length;
                // Add a load operation for the following data in place of the deleted data.
                this.viewItemList.addNextMode(this.viewItemList.createModeLoadItem(false, this.requestCount));
                // Stabilize the scroll position.
                this.viewItemList.stabilizeScroll(false);
            }
        }
        if (!!changes["itemPage"] && !!this.itemPage) {
            let isSetScrollMin = false;
            if (this.isReset) {
                this.viewItemList.clear();
                isSetScrollMin = true;
                this.pageEnd = 0;
            }
            let dataList = this.viewItemList.dataList;
            const rowsOnPage = this.getMinValue(this.rowsOnPage, 0);
            const maxSizeRows = this.getMinValue(this.maxSizeRows, 0);

            let newPageEnd: number | null = null;
            let isAddTop: boolean | null = null;
            if (this.pageBgn - 1 == this.itemPage.page) {
                const delta = (dataList.length + this.itemPage.list.length) > maxSizeRows ? -1 : 0;
                newPageEnd = this.pageEnd + delta;
                isAddTop = true;
            } else if (this.pageBgn == this.itemPage.page) {
                if (dataList.length > 0) {
                    dataList = dataList.slice(rowsOnPage);
                }
                newPageEnd = this.pageEnd;
                isAddTop = true;
            } else if (this.itemPage.page == this.pageEnd) {
                const end = dataList.length > 0 ? (Math.ceil(dataList.length / rowsOnPage) - 1) * rowsOnPage : 0;
                dataList = dataList.slice(0, end);
                newPageEnd = this.pageEnd;
                isAddTop = false;
            } else if (this.itemPage.page == this.pageEnd + 1 || this.pageEnd == 0) {
                newPageEnd = this.itemPage.page;
                isAddTop = false;
            }

            let isUpdateItems: boolean = false;
            if (this.itemPage.list.length > 0 && isAddTop !== null && newPageEnd != null) {
                isUpdateItems = true;
                this.viewItemList.setDataList(
                    this.appendItemPage(dataList, isAddTop, this.itemPage.list, maxSizeRows, rowsOnPage)
                );
                this.pageEnd = newPageEnd;
            }
            let loadCount: number = 0;

            if (this.requestCount > 0) {
                // If requestCount is greater than zero, the data is loaded after the records are deleted.
                const minSizeRows = this.getMinValue(this.minSizeRows, maxSizeRows);
                // Check whether past data needs to be loaded.
                if (isAddTop === false && this.itemPage.list.length == 0 && this.viewItemList.dataList.length < minSizeRows) {
                    // Create parameters to load past data.
                    loadCount = rowsOnPage;
                }
                this.requestCount = 0;
            }

            if (isUpdateItems || isSetScrollMin || loadCount > 0) {
                if (loadCount > 0) {
                    // Add a load operation for past data instead of deleted data.
                    this.viewItemList.addNextMode(this.viewItemList.createModeLoadItem(true, loadCount));
                }
                // Stabilize the scroll position.
                this.viewItemList.stabilizeScroll(isSetScrollMin);
            }
        }
    }

    // ** Public API **

    public loadItemPage = (isAddTop: boolean, item: ItemView, count: number): void => {
        let page = !isAddTop ? this.pageEnd + 1 : this.pageBgn - 1;
        if (this.requestCount > 0 && this.requestCount == count) {
            page = !isAddTop ? this.pageEnd : this.pageBgn;
        }
        this.loadPage.emit({ page, item, count });
    }

    // ** Private API **

    private appendItemPage(list: ItemView[], isAddTop: boolean, itemPage: ItemView[], maxSizeRows: number, rowsOnPage: number): ItemView[] {
        let items: ItemView[] = list;
        if (itemPage.length > 0) {
            items = isAddTop ? itemPage.concat(list) : list.concat(itemPage);
            if (maxSizeRows > 0 && items.length > maxSizeRows) {
                items = isAddTop ? items.slice(0, maxSizeRows) : items.slice(rowsOnPage);
            }
        }
        return items;
    }
    private getMinValue(value: number | null | undefined, minValue: number): number {
        return !!value && value > minValue ? value : minValue;
    }
}
