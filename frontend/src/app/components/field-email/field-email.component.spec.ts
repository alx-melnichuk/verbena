import { ComponentFixture, TestBed } from '@angular/core/testing';

import { FieldEmailComponent } from './field-email.component';

describe('FieldEmailComponent', () => {
  let component: FieldEmailComponent;
  let fixture: ComponentFixture<FieldEmailComponent>;

  beforeEach(() => {
    TestBed.configureTestingModule({
      imports: [FieldEmailComponent],
    });
    fixture = TestBed.createComponent(FieldEmailComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
