import { ComponentFixture, TestBed } from '@angular/core/testing';

import { FieldPassword } from './field-password';

describe('FieldPassword', () => {
  let component: FieldPassword;
  let fixture: ComponentFixture<FieldPassword>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [FieldPassword]
    })
    .compileComponents();

    fixture = TestBed.createComponent(FieldPassword);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
