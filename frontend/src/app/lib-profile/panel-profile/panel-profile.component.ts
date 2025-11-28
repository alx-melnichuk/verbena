import {
    ChangeDetectionStrategy, ChangeDetectorRef, Component, ElementRef, EventEmitter, HostBinding, inject, Input, OnChanges, OnInit, Output,
    SimpleChanges, ViewChild, ViewEncapsulation
} from '@angular/core';
import { CommonModule } from '@angular/common';
import { AbstractControl, FormControl, FormGroup, ReactiveFormsModule, ValidationErrors, ValidatorFn } from '@angular/forms';
import { MatButtonModule } from '@angular/material/button';
import { MatInputModule } from '@angular/material/input';
import { TranslatePipe, TranslateService } from '@ngx-translate/core';

import { IMAGE_VALID_FILE_TYPES, MAX_FILE_SIZE } from 'src/app/common/constants';
import { FieldFileUploadComponent } from 'src/app/components/field-file-upload/field-file-upload.component';
import { FieldImageAndUploadComponent } from 'src/app/components/field-image-and-upload/field-image-and-upload.component';
import {
    EMAIL_MAX_LENGTH, EMAIL_MIN_LENGTH, FieldInputComponent, NICKNAME_MAX_LENGTH, NICKNAME_MIN_LENGTH, NICKNAME_PATTERN
} from 'src/app/components/field-input/field-input.component';
import { FieldLocaleComponent } from 'src/app/components/field-locale/field-locale.component';
import { FieldPasswordComponent } from 'src/app/components/field-password/field-password.component';
import { FieldTextareaComponent } from 'src/app/components/field-textarea/field-textarea.component';
import { FieldThemeComponent } from 'src/app/components/field-theme/field-theme.component';
import { UniquenessCheckComponent } from 'src/app/components/uniqueness-check/uniqueness-check.component';
import { DialogService } from 'src/app/lib-dialog/dialog.service';
import { FileSizeUtil } from 'src/app/utils/file_size.util';
import { HtmlElemUtil } from 'src/app/utils/html-elem.util';
import { ValidFileTypesUtil } from 'src/app/utils/valid_file_types.util';

import { ProfileService } from '../profile.service';
import { ModifyProfileDto, NewPasswordProfileDto, ProfileDto, ProfileDtoUtil, UniquenessDto } from '../profile-api.interface';
import { ProfileConfigDto } from '../profile-config.interface';

export const PPI_DEBOUNCE_DELAY = 900;
export const PPI_AVATAR_MX_HG = '---pp-avatar-mx-hg';
export const PPI_AVATAR_MX_WD = '---pp-avatar-mx-wd';

@Component({
    selector: 'app-panel-profile',
    exportAs: 'appPanelProfile',
    standalone: true,
    imports: [CommonModule, ReactiveFormsModule, MatButtonModule, MatInputModule, TranslatePipe,
        UniquenessCheckComponent, FieldInputComponent, FieldPasswordComponent, FieldTextareaComponent,
        FieldFileUploadComponent, FieldImageAndUploadComponent, FieldThemeComponent, FieldLocaleComponent],
    templateUrl: './panel-profile.component.html',
    styleUrl: './panel-profile.component.scss',
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush
})
export class PanelProfileComponent implements OnInit, OnChanges {
    @Input()
    public profileDto: ProfileDto | null = null;
    @Input()
    public profileConfigDto: ProfileConfigDto | null = null;
    @Input()
    public isDisabledSubmit: boolean = false;
    @Input()
    public errMsgsProfile: string[] = [];
    @Input()
    public errMsgsPassword: string[] = [];
    @Input()
    public errMsgsAccount: string[] = [];

    @ViewChild('fieldNickname', { static: true })
    public fieldNicknameComp!: FieldInputComponent;
    @ViewChild('fieldEmail', { static: true })
    public fieldEmailComp!: FieldInputComponent;

    @Output()
    readonly updateProfile: EventEmitter<{ modifyProfile: ModifyProfileDto, avatarFile: File | null | undefined }> = new EventEmitter();
    @Output()
    readonly updatePassword: EventEmitter<NewPasswordProfileDto> = new EventEmitter();
    @Output()
    readonly deleteAccount: EventEmitter<void> = new EventEmitter();

    @HostBinding('class.global-scroll')
    public get classGlobalScrollVal(): boolean {
        return true;
    }

    public cntlsProfile = {
        nickname: new FormControl('', []),
        email: new FormControl('', []),
        password: new FormControl('', []),
        avatar: new FormControl('', []),
        descript: new FormControl('', []),
        theme: new FormControl('', []),
        locale: new FormControl('', []),
    };
    public formGroupProfile: FormGroup = new FormGroup(this.cntlsProfile);

    public emailMinLen: number = EMAIL_MIN_LENGTH;
    public emailMaxLen: number = EMAIL_MAX_LENGTH;

    public nicknameMinLen: number = NICKNAME_MIN_LENGTH;
    public nicknameMaxLen: number = NICKNAME_MAX_LENGTH;
    public nicknamePattern: string = NICKNAME_PATTERN;

    public cntlsPassword = {
        password: new FormControl(null, []),
        new_password: new FormControl(null, []),
    };
    public formGroupPassword: FormGroup = new FormGroup(this.cntlsPassword);
    public isRequiredPassword: boolean = false;

    public debounceDelay: number = PPI_DEBOUNCE_DELAY;

    public descriptMaxLen = 2048; // 2*1024
    public descriptMinLen = 2;
    // FieldImageAndUpload parameters
    public accepts = IMAGE_VALID_FILE_TYPES;
    public maxSize = MAX_FILE_SIZE;
    public availableFileTypes: string = '';
    public availableMaxFileSize: string = '';

    // FieldImageAndUpload FormControl
    public avatarFile: File | null | undefined;
    public initIsAvatar: boolean = false; // original has an avatar.

    private origProfileDto: ProfileDto = ProfileDtoUtil.create();

    private changeDetector: ChangeDetectorRef = inject(ChangeDetectorRef);
    private dialogService: DialogService = inject(DialogService);
    private profileService: ProfileService = inject(ProfileService);
    private translate: TranslateService = inject(TranslateService);

    constructor(public hostRef: ElementRef<HTMLElement>) {
        this.formGroupPassword.setValidators(this.validatorsForPassword());
    }

    ngOnInit(): void {
        this.cntlsProfile.nickname.markAsTouched();
        this.fieldNicknameComp.markAsTouched();
        this.cntlsProfile.email.markAsTouched();
        this.fieldEmailComp.markAsTouched();
    }

    ngOnChanges(changes: SimpleChanges): void {
        if (!!changes['profileDto']) {
            this.prepareFormGroupByProfileDto(this.profileDto);
            this.cntlsPassword.password.setValue(null);
            this.cntlsPassword.new_password.setValue(null);
            this.isRequiredPassword = false;
            this.formGroupPassword.markAsPristine();
        }
        if (!!changes['profileConfigDto']) {
            this.prepareFormGroupByProfileConfigDto(this.profileConfigDto);
            this.settingProperties(this.hostRef, this.profileConfigDto);
        }
        if (!!changes['isDisabledSubmit']) {
            if (this.isDisabledSubmit != this.formGroupProfile.disabled) {
                this.isDisabledSubmit ? this.formGroupProfile.disable() : this.formGroupProfile.enable();
                this.changeDetector.markForCheck();
            }
            if (this.isDisabledSubmit != this.formGroupPassword.disabled) {
                this.isDisabledSubmit ? this.formGroupPassword.disable() : this.formGroupPassword.enable();
                this.changeDetector.markForCheck();
            }
        }
    }

    // ** Public API **

    // ** Section: Update profile (formGroupProfile) **

    public checkUniquenessNickname = (nickname: string | null | undefined): Promise<boolean> => {
        if (!nickname || this.origProfileDto.nickname.toLowerCase() == nickname.toLowerCase()) {
            return Promise.resolve(true);
        }
        return this.profileService.uniqueness(nickname, '').then((response) => response == null || (response as UniquenessDto).uniqueness);
    }

    public checkUniquenessEmail = (email: string | null | undefined): Promise<boolean> => {
        if (!email || this.origProfileDto.email.toLowerCase() == email.toLowerCase()) {
            return Promise.resolve(true);
        }
        return this.profileService.uniqueness('', email).then((response) => response == null || (response as UniquenessDto).uniqueness);
    }

    public addAvatarFile(file: File): void {
        this.avatarFile = file;
    }
    public deleteAvatarFile(): void {
        this.avatarFile = (!!this.initIsAvatar ? null : undefined);
        if (!!this.initIsAvatar) {
            this.cntlsProfile.avatar.markAsDirty();
        } else {
            this.cntlsProfile.avatar.markAsPristine();
        }
    }

    public updateErrMsgsProfile(errMsgs: string[] = []): void {
        this.errMsgsProfile = errMsgs;
    }

    public saveProfile(formGroup: FormGroup): void {
        if (!formGroup || formGroup.pristine || formGroup.invalid) {
            return;
        }
        const nickname = formGroup.get('nickname')?.value || '';
        const email = formGroup.get('email')?.value || '';
        const descript = formGroup.get('descript')?.value;
        const theme = formGroup.get('theme')?.value;
        const locale = formGroup.get('locale')?.value;

        const modifyProfile: ModifyProfileDto = {
            nickname: (this.origProfileDto.nickname != nickname ? nickname : undefined),
            email: (this.origProfileDto.email != email ? email : undefined),
            descript: (this.origProfileDto.descript != descript ? descript : undefined),
            theme: (this.origProfileDto.theme != theme ? theme : undefined),
            locale: (this.origProfileDto.locale != locale ? locale : undefined),
        };
        const is_all_empty = Object.values(modifyProfile).findIndex((value) => value !== undefined) == -1;
        if (!is_all_empty || this.avatarFile !== undefined) {
            this.updateProfile.emit({ modifyProfile, avatarFile: this.avatarFile });
        }
    }

    // ** Section: Set new password (formPassword) **

    public validatorsForPassword(): ValidatorFn {
        return (control: AbstractControl): ValidationErrors | null => {
            const formGroup = control as FormGroup;
            const cntlPassword = formGroup.get('password');
            const cntlNewPassword = formGroup.get('new_password');
            if (cntlPassword?.pristine) {
                return { pristine: 'password' };
            }
            if (cntlPassword?.invalid) {
                return { invalid: 'password' };
            }
            if (cntlNewPassword?.pristine) {
                return { pristine: 'new_password' };
            }
            if (cntlNewPassword?.invalid) {
                return { invalid: 'new_password' };
            }
            const passwordValue = cntlPassword?.value || '';
            const newPasswordValue = cntlNewPassword?.value || '';
            if (!!passwordValue && !!newPasswordValue && passwordValue == newPasswordValue) {
                return { new_password_equal_to_old_value: true };
            }
            return null;
        };
    }

    public statePassword(passwordValue: string | null) {
        if (this.isRequiredPassword !== !!passwordValue) {
            this.isRequiredPassword = !!passwordValue;
        }
    }

    public checkPassword(formGroup: FormGroup): void {
        if (formGroup.errors != null && formGroup.errors['new_password_equal_to_old_value']) {
            this.errMsgsPassword.push('ExpectationFailed.new_password:equal_to_old_value');
        }
    }

    public setNewPassword(formGroup: FormGroup): void {
        const cntlPassword = formGroup.get('password');
        const cntlNewPassword = formGroup.get('new_password');
        if (formGroup.pristine || formGroup.invalid || !cntlPassword || !cntlNewPassword) {
            return;
        }
        const newPasswordProfileDto: NewPasswordProfileDto = {
            password: cntlPassword.value,
            newPassword: cntlNewPassword.value
        };
        this.updatePassword.emit(newPasswordProfileDto);
    }

    public updateErrMsgsPassword(errMsgs: string[] = []): void {
        this.errMsgsPassword = errMsgs;
    }

    // ** Section "Delete Account" **

    public removeAccount(): void {
        const title = this.translate.instant('panel-profile.dialog_title_question_account');
        const nickname = this.profileDto?.nickname || '';
        const appName = this.translate.instant('app.name');
        const message = this.translate.instant('panel-profile.dialog_message_question_account', { nickname, appName: appName });
        const params = { btnNameCancel: 'buttons.no', btnNameAccept: 'buttons.yes' };
        this.dialogService.openConfirmation(message, title, params, { maxWidth: '40vw' })
            .then((respose) => {
                if (!!respose) {
                    this.deleteAccount.emit();
                }
            });
    }

    // ** Private API **

    private prepareFormGroupByProfileDto(profileDto: ProfileDto | null): void {
        if (!profileDto) {
            return;
        }
        this.origProfileDto = ProfileDtoUtil.create(profileDto);
        this.origProfileDto.descript = (profileDto.descript || '');
        this.origProfileDto.theme = (profileDto.theme || '');
        this.origProfileDto.locale = (profileDto.locale || '');

        Object.freeze(this.origProfileDto);

        this.formGroupProfile.patchValue({
            nickname: profileDto.nickname,
            email: profileDto.email,
            descript: (profileDto.descript || ''),
            theme: (profileDto.theme || ''),
            locale: (profileDto.locale || ''),
            avatar: profileDto.avatar,
        });
        this.avatarFile = undefined;
        this.initIsAvatar = !!profileDto.avatar;
        this.formGroupProfile.markAsPristine();
    }

    private prepareFormGroupByProfileConfigDto(profileConfigDto: ProfileConfigDto | null): void {
        // Set FieldImageAndUpload parameters
        this.maxSize = profileConfigDto?.avatarMaxSize || MAX_FILE_SIZE;
        this.accepts = (profileConfigDto?.avatarValidTypes || []).join(',');
        this.availableFileTypes = ValidFileTypesUtil.text(this.accepts).join(', ').toUpperCase();
        this.availableMaxFileSize = FileSizeUtil.formatBytes(this.maxSize, 1);
    }
    private settingProperties(elem: ElementRef<HTMLElement> | null, profileConfigDto: ProfileConfigDto | null): void {
        const avatarMaxWidth = profileConfigDto?.avatarMaxWidth;
        const maxWidth = (avatarMaxWidth && avatarMaxWidth > 0 ? avatarMaxWidth : undefined)
        HtmlElemUtil.setProperty(elem, PPI_AVATAR_MX_WD, maxWidth?.toString().concat('px'));

        const avatarMaxHeight = profileConfigDto?.avatarMaxHeight;
        const maxHeight = (avatarMaxHeight && avatarMaxHeight > 0 ? avatarMaxHeight : undefined)
        HtmlElemUtil.setProperty(elem, PPI_AVATAR_MX_HG, maxHeight?.toString().concat('px'));
    }
}
