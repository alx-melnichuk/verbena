import { ComponentFixture, TestBed } from '@angular/core/testing';

import { FieldNicknameComponent } from './field-nickname.component';

describe('FieldNicknameComponent', () => {
  let component: FieldNicknameComponent;
  let fixture: ComponentFixture<FieldNicknameComponent>;

  beforeEach(() => {
    TestBed.configureTestingModule({
      imports: [FieldNicknameComponent],
    });
    fixture = TestBed.createComponent(FieldNicknameComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
