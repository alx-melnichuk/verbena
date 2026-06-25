import { ComponentFixture, TestBed } from '@angular/core/testing';

import { ViewItemList } from './view-item-list';

describe('ViewItemList', () => {
  let component: ViewItemList;
  let fixture: ComponentFixture<ViewItemList>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [ViewItemList]
    })
    .compileComponents();

    fixture = TestBed.createComponent(ViewItemList);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
