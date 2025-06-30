import {
    afterNextRender, AfterViewInit, ChangeDetectionStrategy, Component, ElementRef, EventEmitter, inject, Injector, Input,
    OnChanges, Output, SimpleChanges, ViewChild, ViewEncapsulation
} from '@angular/core';
import { CdkTextareaAutosize } from '@angular/cdk/text-field';
import { CommonModule, KeyValue } from '@angular/common';
import { FormControl, FormGroup, ReactiveFormsModule, ValidationErrors } from '@angular/forms';
import { MatButtonModule } from '@angular/material/button';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatInputModule } from '@angular/material/input';
import { MatMenuModule } from '@angular/material/menu';
import { TranslatePipe } from '@ngx-translate/core';

import { DateTimeFormatPipe } from 'src/app/common/date-time-format.pipe';
import { debounceFn } from 'src/app/common/debounce';
import { StringDateTime } from 'src/app/common/string-date-time';
import { SpinnerComponent } from 'src/app/components/spinner/spinner.component';
import { DateUtil } from 'src/app/utils/date.utils';
import { StringDateTimeUtil } from 'src/app/utils/string-date-time.util';
import { ValidatorUtils } from 'src/app/utils/validator.utils';

import { ChatMessageDto, ParamQueryPastMsg } from '../chat-message-api.interface';


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

export const TITLE = 'message';
export const MESSAGE_MAX_ROWS = 3;
export const MESSAGE_MIN_ROWS = 1;
export const MESSAGE_MAX_LENGTH = 255;
export const MESSAGE_MIN_LENGTH = 0;
export const DEBOUNCE_DELAY = 50;
export const MIN_SCROLL_VALUE = 20;

type ChatMsgObj = { [key: number]: number };
type MenuEditMap = Map<number, MenuEdit>;

// <mat-form-field subscriptSizing="dynamic"
// it'll remove the space until an error or hint actually needs to get displayed and only then expands.

@Component({
    selector: 'app-panel-chat',
    standalone: true,
    imports: [CommonModule, ReactiveFormsModule, MatButtonModule, MatFormFieldModule, MatInputModule, MatMenuModule,
        TranslatePipe, DateTimeFormatPipe, SpinnerComponent],
    templateUrl: './panel-chat.component.html',
    styleUrl: './panel-chat.component.scss',
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class PanelChatComponent implements OnChanges, AfterViewInit {
    @Input() // List of new blocked users.
    public blockedUsers: string[] = [];
    @Input() // List of past chat messages.
    public chatPastMsgs: ChatMessageDto[] = [];
    @Input() // List of new chat messages.
    public chatNewMsgs: ChatMessageDto[] = [];
    @Input() // List of IDs of permanently deleted chat messages.
    public chatRmvIds: number[] = [];
    @Input() // Indication that the user is blocked.
    public isBlocked: boolean | null = null;
    @Input() // Indicates that the user can send messages to the chat.
    public isEditable: boolean | null = null;
    @Input() // Indicates that data is being loaded.
    public isLoadData: boolean | null = null;
    @Input() // Indicates that the user is the owner of the chat.
    public isOwner: boolean | null = null;
    @Input()
    public locale: string | null = null;
    @Input()
    public maxLen: number | null = null;
    @Input()
    public minLen: number | null = null;
    @Input()
    public maxRows: number | null = null;
    @Input()
    public minRows: number | null = null;
    @Input()
    public nickname: string | null = null;
    @Input()
    public title = '';

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
    readonly queryPastMsgs: EventEmitter<ParamQueryPastMsg> = new EventEmitter();

    @ViewChild('autosize')
    public autosize!: CdkTextareaAutosize;
    @ViewChild('scrollItem')
    private scrollItemElem!: ElementRef<HTMLElement>;
    @ViewChild('textareaElement')
    public textareaElem!: ElementRef<HTMLTextAreaElement>;

    public chatMsgList: ChatMessageDto[] = [];
    public formControl: FormControl = new FormControl({ value: null, disabled: false }, []);
    public formGroup: FormGroup = new FormGroup({ newMsg: this.formControl });
    public initValue: string | null = null;
    public maxLenVal: number = MESSAGE_MAX_LENGTH;
    public minLenVal: number = MESSAGE_MIN_LENGTH;
    public maxRowsVal: number = MESSAGE_MAX_ROWS;
    public minRowsVal: number = MESSAGE_MIN_ROWS;
    public msgMarked: ChatMessageDto | null = null;
    public msgEditing: ChatMessageDto | null = null;
    public notViewedCount: number = 0;

    readonly blockedUserSet: Set<string> = new Set();
    readonly chatMsgObj: ChatMsgObj = {};
    readonly dbncScrollItem = debounceFn(() => this.doScrollItem(), DEBOUNCE_DELAY);
    readonly formatDate: Intl.DateTimeFormatOptions = { dateStyle: 'medium' };
    readonly formatTime: Intl.DateTimeFormatOptions = { timeStyle: 'short' };
    readonly menuEditMap: MenuEditMap = new Map();

    private get scrollElem(): HTMLElement {
        return this.scrollItemElem.nativeElement;
    }
    private set scrollElem(value: HTMLElement) { }

    private isNoPastData: boolean = false;
    private isNoReactToScroll: boolean = false;
    private lastScrollTop: number = 0;
    private lastScrollBottom: number = 0; // TODO ?
    private smallestDate: StringDateTime | undefined;
    private largestId: number | null = null;

    private readonly _injector = inject(Injector);

    constructor() {
        console.log(`PanelChat();`); // #
        this.prepareFormGroup(this.maxLenVal, this.minLenVal);
    }

    triggerResize() {
        // Wait for content to render, then trigger textarea resize.
        afterNextRender(
            () => { this.autosize.resizeToFitContent(true); },
            { injector: this._injector },
        );
    }

    ngOnChanges(changes: SimpleChanges): void {
        if (!!changes['blockedUsers'] || !!changes['isOwner']) {
            this.blockedUserSet.clear();
            const selfName = this.nickname || ''
            const blockedUsers = this.isOwner ? this.blockedUsers : [];
            for (let idx = 0; idx < blockedUsers.length; idx++) {
                if (selfName != blockedUsers[idx]) {
                    this.blockedUserSet.add(blockedUsers[idx]);
                }
            }
        }
        if (!!changes['chatPastMsgs'] && this.chatPastMsgs.length > 0) {
            // List of past chat messages.
            if (this.chatPastMsgs.length > 0) {
                this.chatMsgList = this.chatPastMsgs.reverse().concat(this.chatMsgList);
                this.updateChatMsgObj(this.chatMsgObj, this.chatMsgList);
                this.smallestDate = this.chatMsgList[0].date;
            } else {
                this.isNoPastData = true;
            }
        }
        /*if (!!changes['chatMsgs']) {
            console.log(`PanelChat.OnChange('chatMsgs') 1 chatMsgs.length: ${this.chatMsgs.length}`);
            this.isNoPastData = this.chatMsgs.length == 0;
            if (this.chatMsgs.length > 0) {
                this.chatMsgList = this.chatMsgs.concat(this.chatMsgList);
                this.updateChatMsgObj(this.chatMsgObj, this.chatMsgList);
            }
            /  *
            let res = null;
            if (this.chatMsgs.length > 0) {
                res = this.loadChatMsgs(this.chatMsgObj, this.chatMsgs, this.menuEditMap, this.nickname || '');
                this.chatMsgList = res.chatMsgs;
            } else {
                this.isNoPastData = true;
            }
            if (!!res && res.smallestId > -1 && res.largestId > -1) {
                const isAddBefore = this.smallestId != null ? res.smallestId < this.smallestId : false;
                const isAddAfter = this.largestId != null ? res.largestId > this.largestId : false;
                const bottom = (isAddBefore && !isAddAfter ? this.lastScrollBottom : (!isAddBefore && isAddAfter ? 0 : -1));
                if (bottom > -1) {
                    Promise.resolve().then(() => this.setItemsScrollTo(this.scrollElem, { bottom }));
                }
                this.smallestId = res.smallestId;
                this.largestId = res.largestId;
            }
            *  /
        }*/
        if (!!changes['chatNewMsgs'] && this.chatNewMsgs.length > 0) {
            const len = this.chatNewMsgs.length;
            // List of new chat messages.
            const newCnt = this.loadNewEdtChatMsgs(this.chatMsgObj, this.chatMsgList, this.chatNewMsgs);
            if (newCnt > 0) {
                if (!this.msgMarked && this.checkScrollingAllowed()) {
                    Promise.resolve().then(() => this.doScrollToLast());
                } else {
                    this.notViewedCount = len;
                }
            }
        }
        if (!!changes['chatRmvIds'] && this.chatRmvIds.length > 0) {
            // List of IDs of permanently deleted chat messages.
            this.chatMsgList = this.loadRmvIds(this.chatMsgList, this.chatRmvIds);
            this.updateChatMsgObj(this.chatMsgObj, this.chatMsgList);
            // Promise.resolve().then(() => this.checkScrollPosition());
        }
        if (!!changes['isEditable'] && !changes['isEditable'].firstChange) {
            Promise.resolve().then(() => this.doScrollToLast());
        }
        if (!!changes['maxLen'] || !!changes['minLen']) {
            this.maxLenVal = (!!this.maxLen && this.maxLen > 0 ? this.maxLen : MESSAGE_MAX_LENGTH);
            this.minLenVal = (!!this.minLen && this.minLen > 0 ? this.minLen : MESSAGE_MIN_LENGTH);
            this.prepareFormGroup(this.maxLenVal, this.minLenVal);
        }
        if (!!changes['maxRows']) {
            this.maxRowsVal = (!!this.maxRows && this.maxRows > 0 ? this.maxRows : MESSAGE_MAX_ROWS);
        }
        if (!!changes['minRows']) {
            this.minRowsVal = (!!this.minRows && this.minRows > 0 ? this.minRows : MESSAGE_MIN_ROWS);
        }
    }
    ngAfterViewInit(): void {
        // this.checkScrollPosition();
    }

    // ** Public API **

    public trackById(index: number, item: ChatMessageDto): number {
        return item.id;
    }
    public doScrollItem(elem: HTMLElement = this.scrollElem): void {
        const isMoveUp = this.lastScrollTop > elem.scrollTop;
        this.lastScrollTop = elem.scrollTop;
        this.lastScrollBottom = elem.scrollHeight - (elem.scrollTop + elem.clientHeight);
        console.log(`_1a doScrollItem(); scrollHeight: ${elem.scrollHeight}, clientHeight: ${elem.clientHeight}`) // #
        /*if (!this.isNoReactToScroll && isMoveUp && !this.isNoPastData && this.deltaScroll(elem) < MIN_SCROLL_VALUE) {
            console.log('_1b this.runQueryPastMsgs();') // #
            this.runQueryPastMsgs();
        }*/
    }
    public doScrollToLast(): void {
        this.setItemsScrollTo(this.scrollElem, { bottom: 0 });
        if (this.notViewedCount > 0) {
            this.notViewedCount = 0;
        }
    }
    public getMenuBlock(nickname: string, isOwner: boolean | null, selfName: string | null): MenuBlock | null {
        const isBlocked = !isOwner ? null : (nickname == selfName ? null : this.blockedUserSet.has(nickname));
        const result = isBlocked != null ? { isBlock: !isBlocked, isUnblock: isBlocked } : null;
        return result;
    }
    public getMenuItem(chatMsg: ChatMessageDto, isOwner: boolean | null, selfName: string | null): MenuItem | null {
        const menuEdit = this.createMenuEdit(selfName || '', chatMsg);
        const menuBlock = this.getMenuBlock(chatMsg.member, isOwner, selfName);
        const result = !!menuEdit || !!menuBlock ? { ...menuEdit, ...menuBlock } : null;
        // console.log(`selfName: ${selfName} getMenuItem(): ${JSON.stringify(result)}`); // #
        return result;
    }
    public cleanNewMsg(): void {
        this.setTextareaValue(null);
        this.msgEditing = null;
    }
    public getErrorMsg(errors: ValidationErrors | null): string {
        return ValidatorUtils.getErrorMsg(errors, TITLE);
    }
    public doSendMessage(newMsg: string): void {
        const newMsgVal = (newMsg || '').trim();
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
    public doSetValueForEditing(chatMsg: ChatMessageDto | null): void {
        if (this.msgEditing != chatMsg) {
            this.msgEditing = chatMsg;
            this.setTextareaValue(chatMsg?.msg || null);
            this.textareaElem.nativeElement.focus();
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
    public doKeydownEnter(event: Event, newMsg: string): void {
        const keyEvent: KeyboardEvent = (event as KeyboardEvent);
        if (this.isEditable && !!newMsg && !keyEvent.altKey && !keyEvent.shiftKey) {
            this.doSendMessage(newMsg);
        }
        event.preventDefault();
    }
    public doKeydownEscape(): void {
        if (!!this.msgEditing) {
            this.cleanNewMsg();
        }
    }
    public doClickCheckSelection(event: Event): void {
        const selectionObj = window.getSelection();
        const selection = !!selectionObj ? selectionObj.toString() : null;
        if (!!selection) {
            event.preventDefault();
            event.stopPropagation();
        }
    }
    // ** **
    /*
    public clickFaceSmilePanel(code: string): void {
        const item: HTMLTextAreaElement = this.getTextArea();
        const start = item.selectionStart;
        const value = item.value;
        item.value = value.substr(0, start) + code + value.substr(start);
        item.selectionStart = start + code.length;
        item.selectionEnd = item.selectionStart;
        this.isShowFaceSmilePanel = false;
        this.messageInput?.focus();
    }*/

    // ** Private API **

    private setTextareaValue(value: string | null): void {
        this.initValue = value;
        this.formControl.setValue(value);
    }
    private prepareFormGroup(maxLen: number, minLen: number): void {
        this.formControl.clearValidators();
        const paramsObj = {
            ...(maxLen > 0 ? { "maxLength": maxLen } : {}),
            ...(minLen > 0 ? { "minLength": minLen } : {}),
        };
        this.formControl.setValidators([...ValidatorUtils.prepare(paramsObj)]);
        this.formControl.updateValueAndValidity();
    }
    private checkScrollPosition(elem: HTMLElement = this.scrollElem): void {
        const isScroll = elem.scrollHeight > elem.clientHeight;
        if (!isScroll && !this.isNoPastData) {
            console.log('_2 this.runQueryPastMsgs();') // #
            // this.runQueryPastMsgs();
        }
    }
    private runQueryPastMsgs(borderDate: StringDateTime | undefined = this.smallestDate): void {
        this.queryPastMsgs.emit({ isSortDes: true, borderDate });
    }
    private deltaScroll(elem: HTMLElement | null | undefined): number {
        let result: number = 0;
        if (!!elem) {
            const height = elem.scrollHeight - elem.clientHeight;
            result = Math.round(Math.round(elem.scrollTop / height * 1000) / 10);
        }
        return result;
    }
    private setItemsScrollTo(elem: HTMLElement | null | undefined, info: { top?: number, bottom?: number }): void {
        let scrollTop = -1;
        if (!!elem && !!info) {
            if (info.top != null && info?.top >= 0) {
                scrollTop = info.top;
            } else if (info.bottom != null && info.bottom >= 0) {
                scrollTop = elem.scrollHeight - (elem.clientHeight + info.bottom);
            }
        }
        if (!!elem && scrollTop > -1) {
            this.isNoReactToScroll = true;
            console.log(`_12a setItemsScrollTo() this.isNoReactToScroll: ${this.isNoReactToScroll}`) // #
            elem.scrollTop = scrollTop;
            setTimeout(() => {
                this.isNoReactToScroll = false;
                console.log(`_12b setItemsScrollTo() this.isNoReactToScroll: ${this.isNoReactToScroll}`) // #
            }, 180);
        }
    }
    /*
    private hexToUtf8(hex: string): string {
        return decodeURIComponent(
            '%' + ((hex || '').match(/.{1,2}/g) || []).join('%')
        );
    }
    */
    private createMenuEdit(selfName: string, chatMsg: ChatMessageDto): MenuEdit | null {
        const isSelfNameEqMember = !!selfName && selfName == chatMsg.member;
        const isEdit = isSelfNameEqMember && !chatMsg.dateRmv;
        const isCut = isEdit;
        const isRemove = isSelfNameEqMember && !!chatMsg.dateRmv;

        return isSelfNameEqMember ? { isEdit, isCut, isRemove } : null;
    }
    private checkScrollingAllowed(elem: HTMLElement = this.scrollElem): boolean {
        const scrollBottom = elem.scrollHeight - (elem.scrollTop + elem.clientHeight);
        console.log(`_3 checkScrollingCapability(): ${scrollBottom < elem.clientHeight}`); // #
        return scrollBottom < elem.clientHeight;
    }
    private loadPastChatMsgs(): void {

    }
    private loadNewEdtChatMsgs(chatMsgObj: ChatMsgObj, chatMsgList: ChatMessageDto[], newEdtMsgs: ChatMessageDto[]): number {
        let result: number = 0;
        for (let idx = 0; idx < newEdtMsgs.length; idx++) {
            const chatMsg = newEdtMsgs[idx];
            if (!chatMsg.dateEdt && !chatMsg.dateRmv) {
                const index = chatMsgList.push(chatMsg) - 1;
                chatMsgObj[chatMsg.id] = index;
                result++;
            } else {
                const index = chatMsgObj[chatMsg.id];
                const chatMsgOld = !!index ? chatMsgList[index] : null;
                if (chatMsgOld?.id == chatMsg.id) {
                    chatMsgList[index] = chatMsg;
                } else {
                    console.log(`Error processing update - id: ${chatMsg.id}`);
                }
            }
        }
        return result;
    }
    private loadRmvChatMsgs(chatMsgObj: ChatMsgObj, chatMsgList: ChatMessageDto[], rmvMsgs: number[]): void {
    }


    private updateChatMsgObj(chatMsgObj: ChatMsgObj, chatMsgs: ChatMessageDto[]): void {
        chatMsgObj = {};
        for (let idx = 0; idx < chatMsgs.length; idx++) {
            chatMsgObj[chatMsgs[idx].id] = idx;
        }
    }
    /*private loadChatMsgs(
        chatMsgObj: ChatMsgObj, chatMsgs: ChatMessageDto[], menuEditMap: MenuEditMap, selfName: string
    ): { chatMsgs: ChatMessageDto[], smallestId: number, largestId: number } {
        for (let idx = 0; idx < chatMsgs.length; idx++) {
            const chatMsg = chatMsgs[idx];
            chatMsgObj[chatMsg.id] = chatMsg;
            const itemMenu: MenuEdit | null = this.createMenuEdit(selfName, chatMsg);
            if (!!itemMenu) {
                menuEditMap.set(chatMsg.id, itemMenu);
            }
        }
        const resChatMsgs = Object.values(chatMsgObj)
        const smallestId = resChatMsgs.length > 0 ? resChatMsgs[0].id : -1;
        const largestId = resChatMsgs.length > 0 ? resChatMsgs[resChatMsgs.length - 1].id : -1;
        return { chatMsgs: resChatMsgs, smallestId, largestId };
    }*/
    private loadRmvIds(chatMsgs: ChatMessageDto[], chatRmvIds: number[]): ChatMessageDto[] {
        let idx0 = 0; const len = chatMsgs.length;
        for (let idx1 = 0; idx1 < len; idx1++) {
            const index = chatRmvIds.length > 0 ? chatRmvIds.indexOf(chatMsgs[idx1].id) : -1;
            if (index > -1) {
                chatRmvIds.splice(index, 1);
            } else {
                if (idx0 < idx1) {
                    chatMsgs[idx0] = chatMsgs[idx1];
                }
                idx0++;
            }
        }
        if (idx0 < len) {
            chatMsgs.splice(idx0, len - idx0);
        }
        return chatMsgs;
    }
}
