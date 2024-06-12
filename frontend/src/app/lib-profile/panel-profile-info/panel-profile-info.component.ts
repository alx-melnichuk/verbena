import {
  ChangeDetectionStrategy, ChangeDetectorRef, Component, Input, OnChanges, SimpleChanges, ViewChild, ViewEncapsulation
} from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormControl, FormGroup, ReactiveFormsModule } from '@angular/forms';
import { MatButtonModule } from '@angular/material/button';
import { TranslateModule } from '@ngx-translate/core';

import { IMAGE_VALID_FILE_TYPES, MAX_FILE_SIZE } from 'src/app/common/constants';
import { debounceFn } from 'src/app/common/debounce';
import { FieldDescriptComponent } from 'src/app/components/field-descript/field-descript.component';
import { FieldEmailComponent    } from 'src/app/components/field-email/field-email.component';
import { FieldFileUploadComponent } from 'src/app/components/field-file-upload/field-file-upload.component';
import { FieldNicknameComponent } from 'src/app/components/field-nickname/field-nickname.component';
import { FieldPasswordComponent } from 'src/app/components/field-password/field-password.component';
import { SpinnerComponent }       from 'src/app/components/spinner/spinner.component';
import { UserDto, UserDtoUtil }   from 'src/app/lib-user/user-api.interface';
import { UserService }            from 'src/app/lib-user/user.service';

const PPI_SPINNER_DIAMETER = 40;

@Component({
  selector: 'app-panel-profile-info',
  standalone: true,
  imports: [CommonModule, ReactiveFormsModule, MatButtonModule, TranslateModule, FieldNicknameComponent, FieldEmailComponent,
    FieldPasswordComponent, FieldDescriptComponent, FieldFileUploadComponent, SpinnerComponent],
  templateUrl: './panel-profile-info.component.html',
  styleUrls: ['./panel-profile-info.component.scss'],
  encapsulation: ViewEncapsulation.None,
  changeDetection: ChangeDetectionStrategy.OnPush
})
export class PanelProfileInfoComponent implements OnChanges {
  @Input()
  public userInfo: UserDto | null = null;
  @Input()
  public isDisabledSubmit: boolean = false;
  @Input()
  public errMsgList: string[] = [];
  
  @ViewChild(FieldNicknameComponent, { static: true })
  public fieldNicknameComp!: FieldNicknameComponent;

  public controls = {
    avatar: new FormControl(null, []),
    nickname: new FormControl(null, []),
    email: new FormControl(null, []),
    password: new FormControl(null, []),
    descript: new FormControl(null, []),
  };
  public formGroup: FormGroup = new FormGroup(this.controls);

  public maxFileSize = MAX_FILE_SIZE;
  public validFileTypes = IMAGE_VALID_FILE_TYPES;
  public addedLogoView: string = '';
  public spinnerDiameter = PPI_SPINNER_DIAMETER;

  public isCheckNickname: boolean = false;
  public errMsgCheckNickname: string | undefined;
  public isCheckEmail: boolean = false;
  public errMsgCheckEmail: string | undefined;

  private origUserDto: UserDto = UserDtoUtil.create();


  constructor(
    private changeDetectorRef: ChangeDetectorRef,
    private userService: UserService
  ) {
  }
    
  ngOnChanges(changes: SimpleChanges): void {
    if (!!changes['userInfo']) {
      this.prepareFormGroupByUserDto(this.userInfo);
    }
  }
 
  // ** Public API **
  
  public checkNickname = debounceFn((nickname) => this.uniquenessNickname(nickname || ''), 1000);

  public addFile(file: File): void {
    // this.addedLogoFile = file;
    // this.controls.logo.setValue(file.name);
    // this.controls.logo.markAsDirty();
  }

  public readFile(buffFile: string[]): void {
    if (buffFile.length > 0) {
      this.addedLogoView = buffFile[1];
    //   this.changeDetectorRef.markForCheck();
    }
  }

  public deleteFileLogo(): void {
    // this.addedLogoFile = (!!this.origLogo ? null : undefined);
    this.addedLogoView = '';
    // this.controls.logo.setValue(null);
    // if (!!this.origLogo) {
    //   this.controls.logo.markAsDirty();
    // } else {
    //   this.controls.logo.markAsPristine();
    // }
  }

  public updateErrMsg(errMsgList: string[] = []): void {
    this.errMsgList = errMsgList;
  }

  public saveProfile(): void {
    console.log(`saveProfile();`);
  }

  public cancelProfile(): void {
    console.log(`cancelProfile();`);
  }

  // ** Private API **

  private prepareFormGroupByUserDto(userInfo: UserDto | null): void {
    if (!userInfo) {
      return;
    }
    this.origUserDto = { ...userInfo };
    Object.freeze(this.origUserDto);

    this.formGroup.patchValue({
      avatar: '',
      nickname: userInfo.nickname,
      email: userInfo.email,
      password: userInfo.password,
      descript: 'Description of a beautiful trip 2024 to greece 6 - E.Allen',
    });
    this.fieldNicknameComp.getFormControl().markAsTouched();
  }

  private uniquenessNickname(nickname: string): void {
    let result = false;
    this.isCheckNickname = true;
    this.uniqueness(nickname, '')
      .then((response) => { result = response != null; })
      .finally(() => {
        this.isCheckNickname = false;
        this.errMsgCheckNickname = (!!result ? 'Conflict.nickname_already_use' : undefined) ;
      });
  }

  private uniqueness(nickname: string, email: string): Promise<unknown> {
    if (this.origUserDto.nickname.toLowerCase() == nickname.toLowerCase()
    || this.origUserDto.email.toLowerCase() == email.toLowerCase()) {
      return Promise.resolve();
    }
    this.changeDetectorRef.markForCheck();
    return this.userService.uniqueness(nickname, email)
      .finally(() => this.changeDetectorRef.markForCheck());
  }
  
}
