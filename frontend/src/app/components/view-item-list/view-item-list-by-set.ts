import { Directive, EventEmitter, inject, Input, OnChanges, Output, SimpleChanges } from "@angular/core";
import { ItemView, ViewItemList } from "./view-item-list";

export interface ItemViewSet {
    list: ItemView[];
    isAddTop: boolean; // true/false - add to the top/bottom of the list;
}

@Directive({
    selector: "app-view-item-list[itemSet]",
    exportAs: "appViewItemListBySet",
    standalone: true,
})
export class ViewItemListBySet implements OnChanges {
    private viewItemList: ViewItemList = inject(ViewItemList);

    @Input()
    public editSet: ItemView[] | null | undefined;
    @Input()
    public deleteIds: unknown[] | null | undefined;
    @Input()
    public isReset: boolean | null | undefined; // Checked only together with "itemSet".
    @Input()
    public itemSet: ItemViewSet | null | undefined;
    @Input()
    public maxSizeRows: number | null | undefined; // The maximum number of rows in the buffer.
    @Input()
    public minSizeRows: number | null | undefined; // The minimum number of rows in the buffer.

    @Output()
    readonly loadSet: EventEmitter<{ isAddTop: boolean, item: ItemView, count: number }> = new EventEmitter();

    private requestCount: number = 0;

    constructor() {
        this.viewItemList.loadItemFn = this.loadItemSet;
    }

    ngOnChanges(changes: SimpleChanges): void {
        if (!!changes["editSet"] && !!this.editSet && this.editSet.length > 0) {
            for (let idx = 0; idx < this.editSet.length; idx++) {
                const item = this.editSet[idx];
                const index = this.viewItemList.dataList.findIndex((val) => val.id == item.id);
                if (index > -1) {
                    this.viewItemList.dataList[index] = item;
                }
            }
        }
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
        if (!!changes["itemSet"] && !!this.itemSet) {
            let isSetScrollMin = false;
            if (this.isReset) {
                this.viewItemList.clear();
                isSetScrollMin = true;
            }
            const maxSizeRows = this.getMinValue(this.maxSizeRows, 0);

            const isUpdateItems = this.itemSet.list.length > 0;
            if (isUpdateItems) {
                this.viewItemList.setDataList(
                    this.appendItemSet(this.viewItemList.dataList, this.itemSet.isAddTop, this.itemSet.list, maxSizeRows)
                );
            }
            let loadCount: number = 0;

            if (this.requestCount > 0) {
                // If requestCount is greater than zero, the data is loaded after the records are deleted.
                const minSizeRows = this.getMinValue(this.minSizeRows, maxSizeRows);
                // Check whether past data needs to be loaded.
                if (this.itemSet.isAddTop === false && this.itemSet.list.length == 0 && this.viewItemList.dataList.length < minSizeRows) {
                    // Create parameters to load past data.
                    loadCount = this.requestCount;
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

    public loadItemSet = (isAddTop: boolean, item: ItemView, count: number): void => {
        this.loadSet.emit({ isAddTop, item, count });
    }

    // ** Private API **

    private appendItemSet(list: ItemView[], isAddTop: boolean, itemSet: ItemView[], maxSizeRows: number): ItemView[] {
        let items: ItemView[] = list;
        if (itemSet.length > 0) {
            items = isAddTop ? itemSet.concat(list) : list.concat(itemSet);
            const delta = items.length - maxSizeRows;
            if (maxSizeRows > 0 && delta > 0) {
                items = isAddTop ? items.slice(0, maxSizeRows) : items.slice(delta);
            }
        }
        return items;
    }
    private getMinValue(value: number | null | undefined, minValue: number): number {
        return !!value && value > minValue ? value : minValue;
    }
}
