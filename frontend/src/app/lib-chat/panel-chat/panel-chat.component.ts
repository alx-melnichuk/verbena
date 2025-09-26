import {
    AfterViewInit, ChangeDetectionStrategy, ChangeDetectorRef, Component, ElementRef, EventEmitter, HostListener,
    inject, Input, OnChanges, Output, SimpleChanges, ViewChild, ViewEncapsulation
} from '@angular/core';
import { CommonModule, KeyValue } from '@angular/common';
import { FormControl, FormGroup, ReactiveFormsModule } from '@angular/forms';
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

import { ChatMessageDto, ParamQueryPastMsg } from '../chat-message-api.interface';
import { FieldMessageComponent } from '../field-message/field-message.component';

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
export const MIN_SCR_TOP_FOR_QUERYING_PAST_MSGS = 15;
export const MIN_SCR_BOT_FOR_RESET_COUNTNOTVIEWED = 30;

type ChatMsgMap = Map<number, number>;

// <mat-form-field subscriptSizing="dynamic"
// it'll remove the space until an error or hint actually needs to get displayed and only then expands.

@Component({
    selector: 'app-panel-chat',
    standalone: true,
    imports: [CommonModule, ReactiveFormsModule, MatButtonModule, MatFormFieldModule, MatInputModule, MatMenuModule,
        TranslatePipe, DateTimeFormatPipe, SpinnerComponent, FieldMessageComponent],
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

    @ViewChild('scrollItem')
    private scrollItem: ElementRef<HTMLElement> | undefined;
    @ViewChild(FieldMessageComponent)
    public fieldMessageComp!: FieldMessageComponent;

    public chatMsgList: ChatMessageDto[] = [];
    public maxLenVal: number = MESSAGE_MAX_LENGTH;
    public minLenVal: number = MESSAGE_MIN_LENGTH;
    public countNotViewed: number = 0;
    public frmCtrlNewMsg = new FormControl<string | null>({ value: null, disabled: false }, []);
    public formGroup: FormGroup = new FormGroup({ newMsg: this.frmCtrlNewMsg });
    public initValue: string | null = null;
    public isFocusMsg: boolean = false;
    public msgMarked: ChatMessageDto | null = null;
    public msgEditing: ChatMessageDto | null = null;
    public pc_debug = false;

    readonly blockedUserSet: Set<string> = new Set();
    readonly chatMsgMap: ChatMsgMap = new Map();
    readonly dbncScrollItem = debounceFn(() => { this.checkScrollBottom(); this.checkScrollTop(); }, DEBOUNCE_DELAY);
    readonly dbncCheckExistScroll = debounceFn(() => { this.checkExistScroll(); }, DEBOUNCE_DELAY);
    readonly formatDate: Intl.DateTimeFormatOptions = { dateStyle: 'medium' };
    readonly formatTime: Intl.DateTimeFormatOptions = { timeStyle: 'short' };

    private isPastMsgsHasEnded: boolean = false; // Flag, previous data has ended.
    private isIgnoreScroll: boolean = false; // Flag to ignore scroll event
    private lastScrollTop: number = 0;
    private smallestDate: StringDateTime | undefined;

    private changeDetector: ChangeDetectorRef = inject(ChangeDetectorRef);

    constructor() {
        const urlParams = new URLSearchParams(window.location.search);
        const pc_debug = urlParams.get('pc_debug');
        this.pc_debug = pc_debug == 'true' || pc_debug == '';
    }

    @HostListener('window:resize', ['$event'])
    handlerResize() {
        this.dbncCheckExistScroll();
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
        if (!!changes['chatPastMsgs']) {
            // List of past chat messages.
            if (this.chatPastMsgs.length > 0) {
                this.chatMsgList = this.loadPastChatMsgs(this.chatMsgMap, this.chatMsgList, this.chatPastMsgs);
                this.smallestDate = this.chatMsgList[0].date;
                this.isIgnoreScroll = true;
                // If the scroll disappears after deleting messages, the "PastMsgs" request is executed.
                // If there is no scroll, then after adding new data, the scroll will appear.
                // In this case, the display of data will not be correct (scrolltop = 0).
                // To correct, scrollBottom = 0.
                if (!this.scrollItem || this.scrollItem.nativeElement.scrollHeight == this.scrollItem.nativeElement.clientHeight) {
                    Promise.resolve().then(() => {
                        this.setScrollBottom(0);
                    });
                }
            } else {
                this.isPastMsgsHasEnded = true;
            }
        }
        if (!!changes['chatNewMsgs'] && this.chatNewMsgs.length > 0) {
            // List of new and edited chat messages.
            const newCnt = this.loadNewEdtChatMsgs(this.chatMsgMap, this.chatMsgList, this.chatNewMsgs).count;
            if (newCnt > 0) {
                if (!this.msgMarked && this.checkScrollingAllowed()) {
                    Promise.resolve().then(() => this.setScrollBottom(0));
                } else {
                    this.countNotViewed = newCnt;
                }
            }
        }
        if (!!changes['chatRmvIds'] && this.chatRmvIds.length > 0) {
            // List of IDs of permanently deleted chat messages.
            this.loadRmvChatMsgs(this.chatMsgMap, this.chatMsgList, this.chatRmvIds);
            const msgMarkedIndex = !!this.msgMarked?.id ? this.chatRmvIds.indexOf(this.msgMarked.id) : -1;
            if (msgMarkedIndex > -1) {
                this.msgMarked = null;
            }
            const msgEditingIndex = !!this.msgEditing?.id ? this.chatRmvIds.indexOf(this.msgEditing.id) : -1;
            if (msgEditingIndex > -1) {
                this.cleanNewMsg();
            }
            Promise.resolve().then(() => this.checkExistScroll());
        }
        if (!!changes['isEditable'] && !changes['isEditable'].firstChange) {
            Promise.resolve().then(() => this.setScrollBottom(0));
        }
        if (!!changes['maxLen']) {
            this.maxLenVal = (!!this.maxLen && this.maxLen > -1 ? this.maxLen : MESSAGE_MAX_LENGTH);
        }
        if (!!changes['minLen']) {
            this.minLenVal = (!!this.minLen && this.minLen > -1 ? this.minLen : MESSAGE_MIN_LENGTH);
        }
    }
    ngAfterViewInit(): void {
        this.checkExistScroll();
    }

    // ** Public API **

    public getMenuBlock(nickname: string, isOwner: boolean | null, selfName: string | null): MenuBlock | null {
        const isBlocked = !isOwner ? null : (nickname == selfName ? null : this.blockedUserSet.has(nickname));
        const result = isBlocked != null ? { isBlock: !isBlocked, isUnblock: isBlocked } : null;
        return result;
    }
    public getMenuItem(chatMsg: ChatMessageDto, isOwner: boolean | null, selfName: string | null): MenuItem | null {
        const menuEdit = this.isEditable ? this.createMenuEdit(selfName || '', chatMsg) : null;
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
        if (this.isEditable && this.msgEditing != chatMsg) {
            this.msgEditing = chatMsg;
            this.setTextareaValue(chatMsg?.msg || null);
            this.fieldMessageComp.focus();
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
        const value1 = value || '';
        return !!value1 && value1 != (original || '');
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
    public checkScrollBottom(elem: HTMLElement | undefined = this.scrollItem?.nativeElement): void {
        const height = !!elem ? elem.scrollHeight - elem.clientHeight : 0;
        if (this.countNotViewed > 0 && !!elem && height > 0 && (height - elem.scrollTop) < MIN_SCR_BOT_FOR_RESET_COUNTNOTVIEWED) {
            this.countNotViewed = 0;
            this.changeDetector.markForCheck();
        }
    }
    public checkScrollTop(elem: HTMLElement | undefined = this.scrollItem?.nativeElement): void {
        if (!!elem) {
            const isMoveUp = this.lastScrollTop > elem.scrollTop;
            this.lastScrollTop = elem.scrollTop;

            if (this.isIgnoreScroll) {
                this.isIgnoreScroll = false;
            } else {
                const height = elem.scrollHeight - elem.clientHeight;
                if (isMoveUp && !this.isPastMsgsHasEnded && height > 0) {
                    const value1 = 0.1405325 * elem.scrollHeight - 78.356;
                    const value2 = Math.round(Math.round(value1 * 100) / 100);
                    let minScrollTop = 50;
                    minScrollTop = value2 > minScrollTop ? value2 : minScrollTop;
                    if (elem.scrollTop < minScrollTop) {
                        if (elem.scrollTop == 0) {
                            this.setScrollTop(2);
                        }
                        this.runQueryPastMsgs();
                    }
                }
            }
        }
    }
    public checkExistScroll(elem: HTMLElement | undefined = this.scrollItem?.nativeElement): void {
        // If the scroll disappears after deleting messages, the "PastMsgs" request is executed.
        if (!!elem && elem.scrollHeight == elem.clientHeight && !this.isPastMsgsHasEnded) {
            this.runQueryPastMsgs();
        }
    }
    public setScrollTop(top: number, elem: HTMLElement | undefined = this.scrollItem?.nativeElement): void {
        if (!!elem) {
            let scrollTop = !!elem && top != null && top >= 0 ? top : -1;
            if (scrollTop > -1) {
                this.isIgnoreScroll = true;
                elem.scrollTop = scrollTop;
            }
        }
    }
    public setScrollBottom(bottom: number, elem: HTMLElement | undefined = this.scrollItem?.nativeElement): void {
        if (!!elem) {
            let scrollTop = !!elem && bottom != null && bottom >= 0 ? elem.scrollHeight - elem.clientHeight - bottom : -1;
            if (scrollTop > -1) {
                this.isIgnoreScroll = true;
                elem.scrollTop = scrollTop;
            }
        }
    }

    // ** Private API **

    private setTextareaValue(value: string | null): void {
        this.initValue = value;
        this.frmCtrlNewMsg.setValue(value);
    }
    private createMenuEdit(selfName: string, chatMsg: ChatMessageDto): MenuEdit | null {
        const isSelfNameEqMember = !!selfName && selfName == chatMsg.member;
        const isEdit = isSelfNameEqMember && !chatMsg.dateRmv;
        const isCut = isEdit;
        const isRemove = isSelfNameEqMember && !!chatMsg.dateRmv;

        return isSelfNameEqMember ? { isEdit, isCut, isRemove } : null;
    }
    private checkScrollingAllowed(elem: HTMLElement | undefined = this.scrollItem?.nativeElement): boolean {
        let result = true;
        if (!!elem) {
            const scrollBottom = elem.scrollHeight - elem.clientHeight - elem.scrollTop;
            result = scrollBottom < elem.clientHeight;
        }
        return result;
    }
    private loadPastChatMsgs(chatMsgMap: ChatMsgMap, chatMsgList: ChatMessageDto[], chatPastMsgs: ChatMessageDto[]): ChatMessageDto[] {
        chatMsgMap.clear();
        const list = chatPastMsgs.reverse().concat(chatMsgList);
        const result: ChatMessageDto[] = [];
        for (let idx = 0; idx < list.length; idx++) {
            const chatMsg = list[idx];
            const index = result.push(chatMsg) - 1;
            chatMsgMap.set(chatMsg.id, index);
        }
        return result;
    }
    private loadNewEdtChatMsgs(
        chatMsgMap: ChatMsgMap, chatMsgList: ChatMessageDto[], chatNewEdtMsgs: ChatMessageDto[]
    ): { count: number, list: ChatMessageDto[] } {
        let count: number = 0;
        for (let idx = 0; idx < chatNewEdtMsgs.length; idx++) {
            const chatMsg = chatNewEdtMsgs[idx];
            if (!chatMsg.dateEdt && !chatMsg.dateRmv) {
                const index = chatMsgList.push(chatMsg) - 1;
                chatMsgMap.set(chatMsg.id, index);
                count++;
            } else {
                const index = chatMsgMap.get(chatMsg.id);
                const chatMsgOld = !!index ? chatMsgList[index] : null;
                if (!!index && chatMsgOld?.id == chatMsg.id) {
                    chatMsgList[index] = chatMsg;
                } else {
                    console.error(`Error processing update - id: ${chatMsg.id}`);
                }
            }
        }
        return { count, list: chatMsgList };
    }
    private loadRmvChatMsgs(chatMsgMap: ChatMsgMap, chatMsgList: ChatMessageDto[], rmvIds: number[]): ChatMessageDto[] {
        let idx0 = 0;
        const len = chatMsgList.length;
        for (let idx1 = 0; idx1 < len; idx1++) {
            const chatMsgId = chatMsgList[idx1].id;
            const index = rmvIds.length > 0 ? rmvIds.indexOf(chatMsgId) : -1;
            if (index > -1) {
                rmvIds.splice(index, 1);
                chatMsgMap.delete(chatMsgId);
            } else {
                if (idx0 < idx1) {
                    chatMsgList[idx0] = chatMsgList[idx1];
                }
                idx0++;
            }
        }
        if (idx0 < len) {
            chatMsgList.splice(idx0, len - idx0);
        }
        return chatMsgList;
    }
    private runQueryPastMsgs(dateLimit: StringDateTime | undefined = this.smallestDate): void {
        this.queryPastMsgs.emit({ isSortDes: true, maxDate: dateLimit });
    }
}
