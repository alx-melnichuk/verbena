import { ComponentFixture, TestBed } from '@angular/core/testing';

import { AlertWrap } from './alert-wrap';

describe('AlertWrap', () => {
  let component: AlertWrap;
  let fixture: ComponentFixture<AlertWrap>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [AlertWrap]
    })
    .compileComponents();

    fixture = TestBed.createComponent(AlertWrap);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
