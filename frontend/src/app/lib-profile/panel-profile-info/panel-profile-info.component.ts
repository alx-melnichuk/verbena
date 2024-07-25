import {
  ChangeDetectionStrategy, Component, EventEmitter, HostBinding, Input, OnChanges, OnInit, Output, SimpleChanges, ViewChild, ViewEncapsulation
} from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormControl, FormGroup, ReactiveFormsModule } from '@angular/forms';
import { MatButtonModule } from '@angular/material/button';
import { TranslateModule } from '@ngx-translate/core';

import { IMAGE_VALID_FILE_TYPES, MAX_FILE_SIZE } from 'src/app/common/constants';
import { FieldDescriptComponent } from 'src/app/components/field-descript/field-descript.component';
import { FieldEmailComponent    } from 'src/app/components/field-email/field-email.component';
import { FieldFileUploadComponent } from 'src/app/components/field-file-upload/field-file-upload.component';
import { FieldNicknameComponent } from 'src/app/components/field-nickname/field-nickname.component';
import { FieldPasswordComponent } from 'src/app/components/field-password/field-password.component';
import { UniquenessCheckComponent } from 'src/app/components/uniqueness-check/uniqueness-check.component';
import { UserDto, UserDtoUtil, UpdateProfileFileDto } from 'src/app/lib-user/user-api.interface';
import { UserService } from 'src/app/lib-user/user.service';
import { FieldImageAndUploadComponent } from 'src/app/components/field-image-and-upload/field-image-and-upload.component';

export const PPI_DEBOUNCE_DELAY = 900;

@Component({
  selector: 'app-panel-profile-info',
  standalone: true,
  imports: [CommonModule, ReactiveFormsModule, MatButtonModule, TranslateModule, FieldNicknameComponent, FieldEmailComponent,
    FieldPasswordComponent, FieldDescriptComponent, FieldFileUploadComponent, FieldImageAndUploadComponent, UniquenessCheckComponent],
  templateUrl: './panel-profile-info.component.html',
  styleUrls: ['./panel-profile-info.component.scss'],
  encapsulation: ViewEncapsulation.None,
  changeDetection: ChangeDetectionStrategy.OnPush
})
export class PanelProfileInfoComponent implements OnInit, OnChanges {
  @Input()
  public userInfo: UserDto | null = null;
  @Input()
  public isDisabledSubmit: boolean = false;
  @Input()
  public errMsgList1: string[] = [];
  @Input()
  public errMsgList2: string[] = [];
  
  @ViewChild(FieldNicknameComponent, { static: true })
  public fieldNicknameComp!: FieldNicknameComponent;
  @ViewChild(FieldEmailComponent, { static: true })
  public fieldEmailComp!: FieldEmailComponent;

  @Output()
  readonly updateProfile: EventEmitter<UpdateProfileFileDto> = new EventEmitter();
  @Output()
  readonly cancelProfile: EventEmitter<void> = new EventEmitter();

  @HostBinding('class.global-scroll')
  public get classGlobalScrollVal(): boolean {
    return true;
  }

  public controls1 = {
    nickname: new FormControl(null, []),
    email: new FormControl(null, []),
    password: new FormControl(null, []),
    avatar: new FormControl('', []),
    descript: new FormControl(null, []),
  };
  public formGroup1: FormGroup = new FormGroup(this.controls1);

  public controls2 = {
    password: new FormControl(null, []),
    new_password: new FormControl(null, []),
  };
  public formGroup2: FormGroup = new FormGroup(this.controls2);

  public debounceDelay: number = PPI_DEBOUNCE_DELAY;
  // Avatar Image Options
  public maxSize = MAX_FILE_SIZE;
  public validTypes = IMAGE_VALID_FILE_TYPES;
  public avatarFile: File | null | undefined;
  public initIsAvatar: boolean = false; // original has an avatar.
  public  avatarView: string = '';
  public  isAvatarOrig: boolean = false; // original has an avatar.
  
  private origUserDto: UserDto = UserDtoUtil.create();

  constructor(
    private userService: UserService
  ) {
  }

  ngOnInit(): void {
    this.fieldNicknameComp.getFormControl().markAsTouched();
    this.fieldEmailComp.getFormControl().markAsTouched();
  }
    
  ngOnChanges(changes: SimpleChanges): void {
    if (!!changes['userInfo']) {
      this.prepareFormGroupByUserDto(this.userInfo);
    }
  }
 
  // ** Public API **
  
  public checkUniquenessNickname = (nickname: string | null | undefined): Promise<boolean> => {
    if (!nickname || this.origUserDto.nickname.toLowerCase() == nickname.toLowerCase()) {
      return Promise.resolve(true);
    }
    return this.userService.uniqueness(nickname, '').then((response) => response == null);
  }

  public checkUniquenessEmail = (email: string | null | undefined): Promise<boolean> => {
    if (!email || this.origUserDto.email.toLowerCase() == email.toLowerCase()) {
      return Promise.resolve(true);
    }
    return this.userService.uniqueness('', email).then((response) => response == null);
  }

  public addAvatarFile(file: File): void {
    this.avatarFile = file;
  }
  public deleteAvatarFile(): void {
    this.avatarFile = (!!this.initIsAvatar ? null : undefined);
    if (!!this.initIsAvatar) {
      this.controls1.avatar.markAsDirty();
    } else {
      this.controls1.avatar.markAsPristine();
    }
  }

  public updateErrMsg1(errMsgList: string[] = []): void {
    this.errMsgList1 = errMsgList;
  }

  public updateErrMsg2(errMsgList: string[] = []): void {
    this.errMsgList2 = errMsgList;
  }

  public saveProfileCard(): void {
    const nickname: string | undefined = this.controls1.nickname.value || undefined;
    const email: string | undefined = this.controls1.email.value || undefined;
    const password: string | undefined = this.controls1.password.value || undefined;
    const descript: string | undefined = this.controls1.descript.value || undefined;

    const updateProfileFileDto: UpdateProfileFileDto = {
      id: this.origUserDto.id,
      nickname: (this.controls1.nickname.dirty ? nickname : undefined),
      email: (this.controls1.email.dirty ? email : undefined),
      password: (this.controls1.password.dirty ? password : undefined),
      descript: (this.controls1.descript.dirty ? descript : undefined),
      avatarFile: this.avatarFile,
    };
    this.updateProfile.emit(updateProfileFileDto);
  }

  public setNewPassword(): void {
    console.log();
  }

  // ** Private API **

  private prepareFormGroupByUserDto(userInfo: UserDto | null): void {
    if (!userInfo) {
      return;
    }
    this.origUserDto = { ...userInfo };
    Object.freeze(this.origUserDto);

    this.formGroup1.patchValue({
      avatar: '/logo/10_02280e22j4di504.png',
      nickname: userInfo.nickname,
      email: userInfo.email,
      password: userInfo.password,
      descript: 'Description of a beautiful trip 2024 to greece 6 - E.Allen',
    });
    this.avatarFile = undefined;
    this.initIsAvatar = !!this.controls1.avatar.value;
     this.avatarView = ''; // /*streamDto.logo ||*/ '/logo/10_02280e22j4di504.png';
     this.isAvatarOrig = !!this.avatarView;
    // this.controls.avatar.disable();
  }  
}
