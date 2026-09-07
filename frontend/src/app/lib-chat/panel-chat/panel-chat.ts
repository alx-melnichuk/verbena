import { CommonModule, KeyValue } from "@angular/common";
import {
    Component, ViewEncapsulation, ChangeDetectionStrategy, OnChanges, Input, Output, EventEmitter, ViewChild, inject, SimpleChanges,
    ChangeDetectorRef
} from "@angular/core";
import { ReactiveFormsModule, FormControl, FormGroup } from "@angular/forms";
import { MatButtonModule } from "@angular/material/button";
import { MatFormFieldModule } from "@angular/material/form-field";
import { MatInputModule } from "@angular/material/input";
import { MatMenuModule } from "@angular/material/menu";
import { MatTooltipModule } from "@angular/material/tooltip";
import { TranslatePipe } from "@ngx-translate/core";
import { DateTimeFormatPipe } from "../../common/date-time-format-pipe";
import { StringDateTime } from "../../common/string-date-time";
import { FieldTextarea } from "../../components/field-textarea/field-textarea";
import { Spinner } from "../../components/spinner/spinner";
import { ItemView, ViewItemList } from "../../components/view-item-list/view-item-list";
import { ViewItemListBySet } from "../../components/view-item-list/view-item-list-by-set";
import { DialogSrv } from "../../lib-dialog/dialog-srv";
import { ItemViewSetMsg } from "../../lib-pg-browse/pg-browse-view/pg-browse-view";
import { DateUtil } from "../../utils/date.utils";
import { StringDateTimeUtil } from "../../utils/string-date-time.util";
import { ChatMessageDto } from "../chat-message";

interface MenuEdit {
    isEdit: boolean;
    isCut: boolean;
    isRemove: boolean;
}
interface MenuBlock {
    isBlock: boolean;
    isUnblock: boolean;
}
interface MenuItem {
    isEdit?: boolean | undefined;
    isCut?: boolean | undefined;
    isRemove?: boolean | undefined;
    isBlock?: boolean | undefined;
    isUnblock?: boolean | undefined;
}

const CN_DEFAULT_LIMIT = 10;

// <mat-form-field subscriptSizing="dynamic"
// it"ll remove the space until an error or hint actually needs to get displayed and only then expands.

@Component({
    selector: "app-panel-chat",
    exportAs: "appPanelChat",
    standalone: true,
    imports: [CommonModule, ReactiveFormsModule, MatButtonModule, MatFormFieldModule, MatInputModule, MatMenuModule, MatTooltipModule,
        DateTimeFormatPipe, FieldTextarea, Spinner, TranslatePipe, ViewItemList, ViewItemListBySet],
    templateUrl: "./panel-chat.html",
    styleUrl: "./panel-chat.scss",
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class PanelChat implements OnChanges {
    private dialogSrv: DialogSrv = inject(DialogSrv);
    private changeDetector: ChangeDetectorRef = inject(ChangeDetectorRef);

    @Input() // List of new blocked users.
    public blockedUsers: string[] = [];
    @Input()
    public deleteIds: number[] = []; // A list of message IDs that have been deleted from the chat.
    @Input() // Indication that the user is blocked.
    public isBlocked: boolean | null | undefined;
    @Input() // Indicates that the user can send messages to the chat.
    public isEditable: boolean | null | undefined;
    @Input() // Indicates that data is being loaded.
    public isLoading: boolean | null | undefined;
    @Input() // Indicates that the user is the owner of the chat.
    public isOwner: boolean | null | undefined;
    @Input()
    public isReset: boolean | null | undefined; // Checked only together with "itemSetMsg".
    @Input()
    public itemSetMsg: ItemViewSetMsg | null | undefined;
    @Input()
    public locale: string | null | undefined;
    @Input()
    public maxLen: number | null | undefined;
    @Input()
    public minLen: number | null | undefined;
    @Input()
    public maxRows: number | null | undefined;
    @Input()
    public minRows: number | null | undefined;
    @Input()
    public maxSizeRows: number | null | undefined = CN_DEFAULT_LIMIT * 3; // The maximum number of rows in the buffer.
    @Input()
    public nickname: string | null | undefined;
    @Input()
    public title: string | null | undefined;

    @Output()
    readonly blockUser: EventEmitter<string> = new EventEmitter();
    @Output()
    readonly unblockUser: EventEmitter<string> = new EventEmitter();
    @Output()
    readonly sendMsg: EventEmitter<string> = new EventEmitter();
    @Output()
    readonly editMsg: EventEmitter<KeyValue<number, string>> = new EventEmitter();
    @Output()
    readonly cutMsg: EventEmitter<KeyValue<number, string>> = new EventEmitter();
    @Output()
    readonly rmvMsg: EventEmitter<number> = new EventEmitter();
    @Output()
    readonly loadSet: EventEmitter<{ isAddTop: boolean, date: StringDateTime | undefined, count: number }> = new EventEmitter();

    @ViewChild(FieldTextarea)
    public fieldTextareaComp!: FieldTextarea;

    @ViewChild(ViewItemList)
    public viewItemList!: ViewItemList;

    private chatDateMax: Date | null = null;
    public editSet: ItemView[] = [];
    public frmCtrlNewMsg = new FormControl<string | null>({ value: null, disabled: false }, []);
    public formGroup: FormGroup = new FormGroup({ newMsg: this.frmCtrlNewMsg });
    public isFocusMsg: boolean = false;
    public isMoveToEnd: boolean = false;
    private isMoveToEndLoad: boolean = false;
    public msgMarked: ChatMessageDto | null = null;
    public msgEditing: ChatMessageDto | null = null;
    public unreadCount: number = 0; // Number of unread messages.
    private unreadRest: number = 0; // There are unread messages on the next page.
    private unreadDateMin: Date | null = null; // Minimum date of unread message.

    readonly blockedUserSet: Set<string> = new Set();
    readonly formatDate: Intl.DateTimeFormatOptions = { dateStyle: "medium" };
    readonly formatTime: Intl.DateTimeFormatOptions = { timeStyle: "short" };
    readonly formatDateTime: Intl.DateTimeFormatOptions = { dateStyle: "medium", timeStyle: "short" };

    ngOnChanges(changes: SimpleChanges): void {
        const selfName = this.nickname || "";
        if (!!changes["blockedUsers"] || !!changes["isOwner"]) {
            this.blockedUserSet.clear();
            const blockedUsers = !!this.isOwner ? this.blockedUsers : [];
            for (let idx = 0; idx < blockedUsers.length; idx++) {
                if (selfName != blockedUsers[idx]) {
                    this.blockedUserSet.add(blockedUsers[idx]);
                }
            }
        }

        if (!!changes["itemSetMsg"] && this.isReset && !!this.itemSetMsg) {
            this.isMoveToEnd = false;
            this.isMoveToEndLoad = false;
            this.unreadCount = 0;
            this.unreadRest = 0;
            this.unreadDateMin = null;
        }

        if (!!changes["itemSetMsg"] && !this.isReset && !!this.itemSetMsg && this.itemSetMsg.list.length > 0) {
            if (this.editSet.length > 0) {
                this.editSet = [];
            }
            const itemList = this.itemSetMsg.list;
            const isAddTop = this.itemSetMsg.isAddTop;

            if (!isAddTop && !this.isMoveToEndLoad && !!this.viewItemList) {
                const maxSizeRows = this.getMinValue(this.maxSizeRows, 0); // Maximum number of records in a buffer.
                const totalRows = this.viewItemList.dataList.length + itemList.length; // Total records in the buffer.
                // If the data packet is from the previous period (itemSetMsg.isAddTop: false)
                // and if the total number of records in the buffer exceeds the maximum,
                // then there will be deletion of records in the buffer.
                if (maxSizeRows > 0 && totalRows > maxSizeRows) {
                    this.isMoveToEndLoad = true; // A sign that the buffer does not have the most recent entries.
                    this.isMoveToEnd = true;
                    this.chatDateMax = StringDateTimeUtil.toDate((this.viewItemList.dataList[0] as ChatMessageDto).date);
                }
            }

            const isMoreTopRows = this.itemSetMsg.isMoreTopRows;

            if (isAddTop && isMoreTopRows === null) {
                const chatMsg = itemList[0] as ChatMessageDto;
                if (this.isMoveToEndLoad || !this.isMoveToEndLoad && this.isMoveToEnd && chatMsg.member != selfName) {
                    const cnt = (itemList as ChatMessageDto[]).filter((val) => !val.dateEdt && !val.dateRmv).length;
                    this.unreadCount += cnt;
                }
            }
            if (isAddTop && !this.isMoveToEndLoad && isMoreTopRows === null && !!this.viewItemList) {
                const chatMsg = itemList[0] as ChatMessageDto;
                if (!this.isMoveToEnd || chatMsg.member == selfName) {
                    // Add a new command to scroll down after loading.
                    this.viewItemList.addNextMode(this.viewItemList.createModeStabilizeScroll(true));
                }
                const { itemsNew, itemsModify } = this.getBothItems(itemList);
                if (itemsModify.length > 0) {
                    this.itemSetMsg.list = itemsNew;
                    this.editSet = itemsModify;
                }
            }

            if (isAddTop && !this.isMoveToEndLoad && isMoreTopRows !== null && this.unreadDateMin !== null) {
                this.unreadDateMin = null;
            }

            const msgDateFrs = StringDateTimeUtil.toDate((itemList[0] as ChatMessageDto)?.date);

            if (isAddTop && this.isMoveToEndLoad && isMoreTopRows !== null && !!msgDateFrs && !!this.chatDateMax) {
                if (this.chatDateMax <= msgDateFrs) {
                    this.isMoveToEndLoad = false;
                    this.chatDateMax = null;
                    this.unreadRest = (this.unreadCount > itemList.length ? this.unreadCount - itemList.length : 0);
                }
            }
            if (isAddTop && this.isMoveToEndLoad && isMoreTopRows === null) {
                this.itemSetMsg.list = [];
                if (this.unreadDateMin == null) {
                    this.unreadDateMin = msgDateFrs;
                }
            }

        }
    }
    private getBothItems(itemList: ItemView[]): { itemsNew: ItemView[], itemsModify: ItemView[] } {
        const itemsNew: ItemView[] = [];
        const itemsModify: ItemView[] = [];
        for (let idx = 0; idx < itemList.length; idx++) {
            const item: ChatMessageDto = itemList[idx] as ChatMessageDto;
            if (!item.dateEdt && !item.dateRmv) {
                itemsNew.push(item);
            } else {
                itemsModify.push(item);
            }
        }
        return { itemsNew, itemsModify };
    }
    // ** Public API **

    public getMenuBlock(nickname: string, isOwner: boolean, selfName: string | null | undefined): MenuBlock | null {
        const isBlocked = !isOwner ? null : (nickname == selfName ? null : this.blockedUserSet.has(nickname));
        const result = isBlocked != null ? { isBlock: !isBlocked, isUnblock: isBlocked } : null;
        return result;
    }
    public getMenuItem(chatMsg: ChatMessageDto, isOwner: boolean, selfName: string | null | undefined): MenuItem | null {
        const menuEdit = this.isEditable ? this.createMenuEdit(selfName || "", chatMsg) : null;
        const menuBlock = this.getMenuBlock(chatMsg.member, isOwner, selfName);
        const result = !!menuEdit || !!menuBlock ? { ...menuEdit, ...menuBlock } : null;
        return result;
    }
    public cleanNewMsg(): void {
        this.setTextareaValue(null);
        if (!!this.msgEditing) {
            this.msgEditing = null;
        }
    }
    public doSendMessage(newMsg: string | null): void {
        const newMsgVal = (newMsg || "").trim();
        if (this.isEditable && newMsgVal.length > 0) {
            if (!!this.msgEditing && this.msgEditing.id > 0 && !this.msgEditing.dateRmv) {
                const keyValue: KeyValue<number, string> = { key: this.msgEditing.id, value: newMsgVal };
                this.editMsg.emit(keyValue);
            } else {
                this.sendMsg.emit(newMsgVal);
            }
            this.cleanNewMsg();
        }
    }
    public doCutMessage(chatMsg: ChatMessageDto | null): void {
        if (this.isEditable && !!chatMsg && !!chatMsg.id && chatMsg.member == this.nickname && !chatMsg.dateRmv) {
            const keyValue: KeyValue<number, string> = { key: chatMsg.id, value: chatMsg.msg };
            this.cutMsg.emit(keyValue);
        }
    }
    public doRemoveMessage(chatMsg: ChatMessageDto | null): void {
        if (this.isEditable && !!chatMsg && !!chatMsg.id && chatMsg.member == this.nickname && !!chatMsg.dateRmv) {
            this.rmvMsg.emit(chatMsg.id);
        }
    }
    public doResetValueForEditing(): void {
        if (this.isEditable) {
            this.msgEditing = null;
            this.setTextareaValue(null);
            this.fieldTextareaComp.focus();
        }
    }
    public doSetValueForEditing(chatMsg: ChatMessageDto | null): void {
        if (this.isEditable && this.msgEditing != chatMsg) {
            this.checkForEdit(this.frmCtrlNewMsg.value)
                .then((isEditing) => {
                    if (isEditing) {
                        this.msgEditing = chatMsg;
                        this.setTextareaValue(chatMsg?.msg || null);
                        this.fieldTextareaComp.focus();
                    }
                });
        }
    }
    public doBlockUser(member: string | null | undefined, blockedUsers: string[] | null): void {
        if (!!member && !!blockedUsers && !blockedUsers.includes(member)) {
            this.blockUser.emit(member);
        }
    }
    public doUnblockUser(member: string | null | undefined, blockedUsers: string[] | null): void {
        if (!!member && !!blockedUsers && blockedUsers.includes(member)) {
            this.unblockUser.emit(member);
        }
    }
    public isSelf(nickname: string): boolean {
        return (this.nickname === nickname);
    }
    public isToday(value: StringDateTime | null | undefined): boolean {
        let result: boolean = false;
        if (!!value && value.length > 0) {
            result = DateUtil.compare(StringDateTimeUtil.toDate(value), new Date(Date.now())) == 0;
        }
        return result;
    }
    public checkValueForNotEmptyAndChanges(value: string | undefined | null, original: string | null): boolean {
        const value1 = value || "";
        return !!value1 && value1 != (original || "");
    }
    public doKeydownEnter(event: Event, newMsg: string | null): void {
        const keyEvent: KeyboardEvent = (event as KeyboardEvent);
        if (this.isEditable && !!newMsg && !keyEvent.altKey && !keyEvent.shiftKey) {
            this.doSendMessage(newMsg);
        }
        event.preventDefault();
    }
    public doClickCheckSelection(event: Event): void {
        const selectionObj = window.getSelection();
        const selection = !!selectionObj ? selectionObj.toString() : null;
        if (!!selection) {
            event.preventDefault();
            event.stopPropagation();
        }
    }
    public doLoadSetChatMsg(isAddTop: boolean, item: ItemView | undefined, count: number): void {
        const date: string | undefined = !!item ? (item as ChatMessageDto).date : undefined;
        this.loadSet.emit({ isAddTop, date, count });
    }
    public doAfterScroll(isModifiedList: boolean, ratioY: number): void {
        if (!isModifiedList && this.isMoveToEnd && !this.isMoveToEndLoad && 0 <= ratioY
            && 0 < this.unreadCount && this.unreadDateMin == null) {
            const idx = Math.round(ratioY * this.viewItemList.dataList.length);
            const unreadIdx = this.unreadCount - this.unreadRest;
            if (idx < unreadIdx) {
                this.unreadCount = idx + this.unreadRest;
                this.changeDetector.markForCheck();
            }
        }

        if (!isModifiedList && !this.isMoveToEnd) {
            if (0.1 <= ratioY && ratioY < 1) {
                this.isMoveToEnd = true;
                this.changeDetector.markForCheck();
            }
        } else if (!isModifiedList && this.isMoveToEnd) {
            if (0 <= ratioY && ratioY < 0.1 && !this.isMoveToEndLoad && 0 == this.unreadCount) {
                this.isMoveToEnd = false;
                this.changeDetector.markForCheck();
            }
        }
    }
    public doMoveToEndOfList(): void {
        this.isMoveToEnd = false;
        this.unreadCount = 0;
        this.unreadRest = 0;
        this.changeDetector.markForCheck();
        if (!this.isMoveToEndLoad) {
            this.viewItemList.setScrollBottom(0);
        } else {
            this.isMoveToEndLoad = false;
            Promise.resolve().then(() => this.doLoadSetChatMsg(false, undefined, 0));
        }
    }

    // ** Private API **

    private setTextareaValue(value: string | null): void {
        this.frmCtrlNewMsg.setValue(value);
    }
    private createMenuEdit(selfName: string, chatMsg: ChatMessageDto): MenuEdit | null {
        const isSelfNameEqMember = !!selfName && selfName == chatMsg.member;
        const isEdit = isSelfNameEqMember && !chatMsg.dateRmv;
        const isCut = isEdit;
        const isRemove = isSelfNameEqMember && !!chatMsg.dateRmv;

        return isSelfNameEqMember ? { isEdit, isCut, isRemove } : null;
    }
    private checkForEdit(newMsg: string | null): Promise<boolean> {
        const newMsgVal = (newMsg || "").trim();
        if (this.isEditable && newMsgVal.length > 0) {
            return this.dialogSrv.openConfirmation("panel-chat.msg_discard_draft", "dialog.confirmation",
                { btnNameCancel: "buttons.no", btnNameAccept: "buttons.yes" }).then((res) => !!res);
        } else {
            return Promise.resolve(true);
        }
    }
    private getMinValue(value: number | null | undefined, minValue: number): number {
        return !!value && value > minValue ? value : minValue;
    }
}
